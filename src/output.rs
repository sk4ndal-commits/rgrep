use std::fmt::Write as _;

pub fn append_formatted_line(out: &mut String, _filename: Option<&str>, idx: usize, line: &str, _is_match: bool, _line_mode: bool) {
    // Always prefix with 1-based line number, not filename
    let line_no = idx + 1;
    let _ = writeln!(out, "{}:{}", line_no, line);
}
