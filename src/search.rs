use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::Read;
use rayon::prelude::*;

use crate::config::{Config, ExitStatus, RunResult};
use crate::io_utils::{open_input, read_to_lines};
use crate::output::append_formatted_line;
use crate::regex_utils::{build_regex, highlight_segments};
use crate::fs_utils::{expand_inputs, is_binary_path};

pub fn run_on_reader<R: Read>(cfg: &Config, mut reader: R, name: Option<&str>) -> Result<RunResult, String> {
    if cfg.patterns.is_empty() {
        return Err("no pattern provided".into());
    }

    let re = build_regex(cfg).map_err(|e| e.to_string())?;

    let lines = read_to_lines(&mut reader).map_err(|e| e.to_string())?;

    let mut out = String::new();
    let mut matched_any = false;

    let mut before_buf: VecDeque<(usize, String)> = VecDeque::new();
    let mut after_remaining = 0usize;

    let show_filename = name.is_some();

    let mut match_count = 0usize;

    for (idx, raw_line) in lines.iter().enumerate() {
        let is_match = re.is_match(raw_line);
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
            if cfg.color && !cfg.line { // even if -x, we'll highlight entire line when it matches; but to be precise, highlight matches
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

    if cfg.count {
        if !cfg.quiet {
            if show_filename {
                let _ = writeln!(&mut out, "{}:{}", name.unwrap(), match_count);
            } else {
                let _ = writeln!(&mut out, "{}", match_count);
            }
        }
    }

    let status = if matched_any { ExitStatus::MatchFound } else { ExitStatus::NoMatch };

    if cfg.quiet {
        Ok(RunResult { output: String::new(), status })
    } else {
        Ok(RunResult { output: out, status })
    }
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
        return Ok(RunResult { output: String::new(), status: ExitStatus::NoMatch });
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

    let mut out = String::new();
    let mut matched_any = false;
    let mut errs: Vec<String> = Vec::new();

    let mut results_sorted = results;
    results_sorted.sort_by_key(|(i, _)| *i);

    for (_, res) in results_sorted {
        match res {
            Ok(rr) => {
                if !cfg.quiet {
                    out.push_str(&rr.output);
                }
                if rr.status == ExitStatus::MatchFound {
                    matched_any = true;
                }
            }
            Err(e) => errs.push(e),
        }
    }

    if !errs.is_empty() {
        return Err(errs.join("\n"));
    }

    let status = if matched_any { ExitStatus::MatchFound } else { ExitStatus::NoMatch };
    Ok(RunResult { output: if cfg.quiet { String::new() } else { out }, status })
}
