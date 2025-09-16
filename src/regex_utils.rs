//! Regex construction and highlighting utilities.
//!
//! These helpers build a unified Regex from the provided patterns and options,
//! and provide simple ANSI color highlighting of match segments in a line.

use colored::{Colorize, ColoredString};
use regex::{Regex, RegexBuilder};

use crate::config::Config;

/// Build a Regex from `cfg.patterns` honoring word/line, case, and dotall options.
///
/// Multiple patterns are combined with alternation ("|"). Word and line constraints
/// are applied by wrapping the pattern. Multi-line mode is enabled to allow "^"/"$"
/// to match per-line.
pub fn build_regex(cfg: &Config) -> Result<Regex, regex::Error> {
    // Combine multiple patterns using alternation
    let mut pat = cfg
        .patterns
        .iter()
        .map(|p| p.as_str())
        .collect::<Vec<_>>()
        .join("|");

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
