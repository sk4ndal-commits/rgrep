use rayon::prelude::*;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::Read;

use crate::config::{Config, ExitStatus, RunResult};
use crate::fs_utils::{expand_inputs, is_binary_path};
use crate::io_utils::{open_input, read_to_lines};
use crate::output::append_formatted_line;
use crate::regex_utils::{build_regex, highlight_segments, parse_boolean_if_complex};
use regex::Regex;

/// Run a search over any `Read` implementor (e.g., a file, stdin, or in-memory buffer).
///
/// - `cfg` controls the search behavior (patterns, flags, context, etc.).
/// - `reader` provides the input text.
/// - `name` is an optional filename used for prefixes in the formatted output. When `None`,
///   no filename prefix is added and line numbers start at 1.
///
/// Returns a `RunResult` with formatted output (unless `quiet`) and an `ExitStatus` indicating
/// whether any match was found.
pub fn run_on_reader<R: Read>(
    cfg: &Config,
    mut reader: R,
    name: Option<&str>,
) -> Result<RunResult, String> {
    if cfg.patterns.is_empty() {
        return Err("no pattern provided".into());
    }

    // Check for complex Boolean expressions first
    let boolean_expr = parse_boolean_if_complex(cfg).map_err(|e| e.to_string())?;

    let (re, and_matchers) = if boolean_expr.is_some() {
        // For Boolean expressions, we still need a regex for highlighting
        // Use a simple OR of all patterns for highlighting
        (build_regex(cfg).map_err(|e| e.to_string())?, None)
    } else {
        // Use existing logic
        let re = build_regex(cfg).map_err(|e| e.to_string())?;
        let and_matchers =
            crate::regex_utils::build_and_matchers(cfg).map_err(|e| e.to_string())?;
        (re, and_matchers)
    };

    let lines = read_to_lines(&mut reader).map_err(|e| e.to_string())?;

    let mut out = String::new();
    let mut matched_any = false;

    let mut before_buf: VecDeque<(usize, String)> = VecDeque::new();
    let mut after_remaining = 0usize;

    let show_filename = name.is_some();

    let mut match_count = 0usize;

    for (idx, raw_line) in lines.iter().enumerate() {
        let is_match = if let Some((ref expr, ref regexes)) = boolean_expr {
            // Use Boolean expression evaluation
            expr.matches(raw_line, regexes)
        } else if let Some(ref ands) = and_matchers {
            // Use existing AND logic
            ands.iter().all(|r| r.is_match(raw_line))
        } else {
            // Use simple regex matching
            re.is_match(raw_line)
        };
        let final_match = if cfg.invert { !is_match } else { is_match };

        if final_match {
            matched_any = true;
            match_count += 1;
        }

        if cfg.count {
            // Only counting; continue processing to get per-file total
            // reset context buffers appropriately
            after_remaining = cfg.context.after; // for consistency though not used in count
        } else if final_match {
            // Print context before
            if cfg.context.before > 0 {
                while let Some((bidx, bline)) = before_buf.pop_front() {
                    append_formatted_line(&mut out, name, bidx, &bline, false, false);
                }
            }
            // Print the matching line
            if cfg.color && !cfg.line {
                // even if -x, we'll highlight entire line when it matches; but to be precise, highlight matches
                let hl = highlight_segments(raw_line, &re);
                append_formatted_line(&mut out, name, idx, &hl, true, cfg.line);
            } else {
                append_formatted_line(&mut out, name, idx, raw_line, true, cfg.line);
            }

            // Prepare after-context printing for next lines
            after_remaining = cfg.context.after;
        } else {
            // Non-matching line; manage before/after buffers
            if cfg.context.before > 0 {
                before_buf.push_back((idx, raw_line.clone()));
                if before_buf.len() > cfg.context.before {
                    before_buf.pop_front();
                }
            }

            if after_remaining > 0 {
                append_formatted_line(&mut out, name, idx, raw_line, false, false);
                after_remaining -= 1;
            }
        }

        // Separator between groups when both before and after contexts are used
        if after_remaining == 0 && !out.is_empty() {
            // No separators for simplicity (GNU grep uses -- between groups)
        }
    }

    if cfg.count && !cfg.quiet {
        if show_filename {
            let _ = writeln!(&mut out, "{}:{}", name.unwrap(), match_count);
        } else {
            let _ = writeln!(&mut out, "{}", match_count);
        }
    }

    let status = if matched_any {
        ExitStatus::MatchFound
    } else {
        ExitStatus::NoMatch
    };

    if cfg.quiet {
        Ok(RunResult {
            output: String::new(),
            status,
        })
    } else {
        Ok(RunResult {
            output: out,
            status,
        })
    }
}

/// Run a search across input files/paths.
///
/// - If `inputs` contains a single "-", stdin is read.
/// - Directories are traversed when `cfg.recursive` is set.
/// - Binary files are skipped.
/// - With a single file and `cfg.count = true`, the output omits the filename prefix.
///
/// Returns a `RunResult` with aggregated formatted output (unless `quiet`) and combined
/// `ExitStatus` reflecting whether any match was found across all inputs.
fn parse_ts_from_formatted_line(line: &str) -> Option<(i32, i32, i32, i32, i32, i32, i32)> {
    // Expect formatted line like "<lineno>:<content>". We parse timestamp from content.
    let content = match line.splitn(2, ':').nth(1) {
        Some(s) => s,
        None => return None,
    };
    // Regex for timestamps: YYYY-MM-DD[ T]HH:MM:SS(.fraction)? (timezone ignored)
    // Compile regex on each call; acceptable for minimal change. Pattern kept simple.
    let re = Regex::new(r"(?x)
        (?P<y>\d{4})-
        (?P<m>\d{2})-
        (?P<d>\d{2})
        [ T]
        (?P<h>\d{2}):
        (?P<min>\d{2}):
        (?P<s>\d{2})
        (?:\.(?P<frac>\d{1,9}))?
    ").ok()?;
    if let Some(caps) = re.captures(content) {
        let y: i32 = caps.name("y")?.as_str().parse().ok()?;
        let m: i32 = caps.name("m")?.as_str().parse().ok()?;
        let d: i32 = caps.name("d")?.as_str().parse().ok()?;
        let h: i32 = caps.name("h")?.as_str().parse().ok()?;
        let min: i32 = caps.name("min")?.as_str().parse().ok()?;
        let s: i32 = caps.name("s")?.as_str().parse().ok()?;
        let frac_str = caps.name("frac").map(|m| m.as_str()).unwrap_or("");
        let mut nanos: i32 = 0;
        if !frac_str.is_empty() {
            // Normalize to nanoseconds by right-padding with zeros up to 9 digits
            let mut ns = String::from(frac_str);
            while ns.len() < 9 { ns.push('0'); }
            // Truncate if more than 9
            let ns = &ns[..9];
            nanos = ns.parse().unwrap_or(0);
        }
        return Some((y, m, d, h, min, s, nanos));
    }
    None
}

// Helper: Collect all lines with optional parsed timestamps from per-file outputs.
fn collect_all_lines(outputs_per_file: &[(usize, String)]) -> Vec<(Option<(i32,i32,i32,i32,i32,i32,i32)>, usize, usize, String)> {
    let mut all_lines: Vec<(Option<(i32,i32,i32,i32,i32,i32,i32)>, usize, usize, String)> = Vec::new();
    for (file_idx, s) in outputs_per_file {
        for (line_idx, line) in s.lines().enumerate() {
            let ts = parse_ts_from_formatted_line(line);
            all_lines.push((ts, *file_idx, line_idx, line.to_string()));
        }
    }
    all_lines
}

// Helper: Merge lines chronologically if every line has a timestamp; otherwise return None.
fn merge_chronologically(mut all_lines: Vec<(Option<(i32,i32,i32,i32,i32,i32,i32)>, usize, usize, String)>) -> Option<String> {
    if all_lines.is_empty() || all_lines.iter().any(|(ts, _, _, _)| ts.is_none()) {
        return None;
    }
    all_lines.sort_by(|a, b| {
        let ka = a.0.unwrap();
        let kb = b.0.unwrap();
        let ord = ka.cmp(&kb);
        if ord != std::cmp::Ordering::Equal { return ord; }
        let ord2 = a.1.cmp(&b.1);
        if ord2 != std::cmp::Ordering::Equal { return ord2; }
        a.2.cmp(&b.2)
    });
    let mut merged = String::new();
    for (_, _, _, line) in all_lines {
        merged.push_str(&line);
        merged.push('\n');
    }
    Some(merged)
}

// Helper: Concatenate outputs in input order.
fn concat_outputs(outputs_per_file: Vec<(usize, String)>) -> String {
    let mut out = String::new();
    for (_idx, s) in outputs_per_file {
        out.push_str(&s);
    }
    out
}

pub fn run(cfg: &Config, inputs: &[String]) -> Result<RunResult, String> {
    let files = expand_inputs(cfg, inputs);
    if files.len() == 1 && files[0] == "-" {
        let reader = std::io::stdin();
        return run_on_reader(cfg, reader, None);
    }

    // Filter out binary files
    let files: Vec<(usize, String)> = files
        .into_iter()
        .enumerate()
        .filter(|(_, f)| !is_binary_path(f))
        .collect();

    if files.is_empty() {
        return Ok(RunResult {
            output: String::new(),
            status: ExitStatus::NoMatch,
        });
    }

    if files.len() == 1 {
        let name = files[0].1.clone();
        let reader = open_input(Some(&name)).map_err(|e| e.to_string())?;
        if cfg.count {
            // For count-only with a single file, suppress filename prefix
            return run_on_reader(cfg, reader, None);
        } else {
            return run_on_reader(cfg, reader, Some(&name));
        }
    }

    // Parallel processing across files; preserve input order in aggregation
    let results: Vec<(usize, Result<RunResult, String>)> = files
        .par_iter()
        .map(|(idx, name)| {
            let res = open_input(Some(name))
                .map_err(|e| e.to_string())
                .and_then(|rdr| run_on_reader(cfg, rdr, Some(name)));
            (*idx, res)
        })
        .collect();

    let mut matched_any = false;
    let mut errs: Vec<String> = Vec::new();

    let mut outputs_per_file: Vec<(usize, String)> = Vec::new();

    let mut results_sorted = results;
    results_sorted.sort_by_key(|(i, _)| *i);

    for (file_idx, res) in results_sorted {
        match res {
            Ok(rr) => {
                if rr.status == ExitStatus::MatchFound {
                    matched_any = true;
                }
                outputs_per_file.push((file_idx, rr.output));
            }
            Err(e) => errs.push(e),
        }
    }

    if !errs.is_empty() {
        return Err(errs.join("\n"));
    }

    // If quiet, no need to build output at all
    if cfg.quiet {
        let status = if matched_any { ExitStatus::MatchFound } else { ExitStatus::NoMatch };
        return Ok(RunResult { output: String::new(), status });
    }

    // In count mode, just concatenate as-before (no chronological meaning)
    if cfg.count {
        let out = concat_outputs(outputs_per_file);
        let status = if matched_any { ExitStatus::MatchFound } else { ExitStatus::NoMatch };
        return Ok(RunResult { output: out, status });
    }

    // Try to chronologically merge lines across files by timestamp in the content.
    let all_lines = collect_all_lines(&outputs_per_file);
    let out = match merge_chronologically(all_lines) {
        Some(merged) => merged,
        None => concat_outputs(outputs_per_file),
    };

    let status = if matched_any { ExitStatus::MatchFound } else { ExitStatus::NoMatch };
    Ok(RunResult { output: out, status })
}
