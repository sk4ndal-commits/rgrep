//! Output formatting helpers.
//!
//! Currently we always prefix lines with a 1-based line number. Filename prefixes
//! are intentionally omitted for simplicity, except in count mode for multi-file
//! searches where aggregation occurs elsewhere.

use std::fmt::Write as _;

/// Append a single formatted line to the output buffer.
///
/// Parameters:
/// - `out`: destination buffer
/// - `_filename`: optional filename (ignored in current formatting)
/// - `idx`: zero-based line index; will be printed as one-based
/// - `line`: the line content without trailing newline
/// - `_is_match`: whether the line is a primary match (currently unused here)
/// - `_line_mode`: whether whole-line matching is active (unused here)
pub fn append_formatted_line(out: &mut String, _filename: Option<&str>, idx: usize, line: &str, _is_match: bool, _line_mode: bool) {
    // Always prefix with 1-based line number, not filename
    let line_no = idx + 1;
    let _ = writeln!(out, "{}:{}", line_no, line);
}
