//! Regex construction and highlighting utilities.
//!
//! These helpers build a unified Regex from the provided patterns and options,
//! and provide simple ANSI color highlighting of match segments in a line.

use colored::{Colorize, ColoredString};
use regex::{Regex, RegexBuilder};

use crate::config::Config;

fn split_unescaped(input: &str, sep: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut escaped = false;
    for ch in input.chars() {
        if escaped {
            cur.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            cur.push(ch); // keep backslash in pattern
            escaped = true;
            continue;
        }
        if ch == sep {
            parts.push(cur);
            cur = String::new();
        } else {
            cur.push(ch);
        }
    }
    parts.push(cur);
    parts
}

/// Build a Regex from `cfg.patterns` honoring word/line, case, and dotall options.
///
/// When the single provided pattern contains '&', it is treated as an AND-expression; for
/// highlighting we build an alternation of the individual terms. Otherwise, the pattern is
/// used as-is (multiple `|` inside are treated by the regex engine).
pub fn build_regex(cfg: &Config) -> Result<Regex, regex::Error> {
    let raw = cfg.patterns.join("");
    let parts = if raw.contains('&') { Some(split_unescaped(&raw, '&')) } else { None };

    let mut pat = if let Some(ps) = &parts {
        ps.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("|")
    } else {
        raw
    };

    // Wrap for word/line constraints
    if cfg.word {
        pat = format!("\\b(?:{})\\b", pat);
    }
    if cfg.line {
        pat = format!("^(?:{})$", pat);
    }

    let mut builder = RegexBuilder::new(&pat);
    builder.multi_line(true);
    if cfg.case_insensitive { builder.case_insensitive(true); }
    if cfg.dotall { builder.dot_matches_new_line(true); }
    builder.build()
}

/// Build regexes for AND parts if '&' is present; otherwise return None.
pub fn build_and_matchers(cfg: &Config) -> Result<Option<Vec<Regex>>, regex::Error> {
    let raw = cfg.patterns.join("");
    if !raw.contains('&') {
        return Ok(None);
    }
    let parts = split_unescaped(&raw, '&');
    let mut regs = Vec::with_capacity(parts.len());
    for mut p in parts {
        if cfg.word { p = format!("\\b(?:{})\\b", p); }
        // Do NOT apply ^...$ for -x here; AND of full-line matches is nearly always impossible.
        let mut b = RegexBuilder::new(&p);
        b.multi_line(true);
        if cfg.case_insensitive { b.case_insensitive(true); }
        if cfg.dotall { b.dot_matches_new_line(true); }
        regs.push(b.build()?);
    }
    Ok(Some(regs))
}

pub fn highlight_segments(line: &str, re: &Regex) -> String {
    let mut result = String::with_capacity(line.len() + 16);
    let mut last = 0;
    for m in re.find_iter(line) {
        let (s, e) = (m.start(), m.end());
        if s > last {
            result.push_str(&line[last..s]);
        }
        let seg: ColoredString = line[s..e].to_string().red().bold();
        result.push_str(&seg.to_string());
        last = e;
    }
    if last < line.len() {
        result.push_str(&line[last..]);
    }
    result
}
