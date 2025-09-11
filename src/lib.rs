use colored::{Colorize, ColoredString};
use regex::{Regex, RegexBuilder};
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub before: usize,
    pub after: usize,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub patterns: Vec<String>,
    pub invert: bool,   // -v
    pub count: bool,    // -c
    pub quiet: bool,    // -q
    pub word: bool,     // -w
    pub line: bool,     // -x
    pub context: Context, // -A, -B, -C
    pub color: bool,    // syntax highlighting
}

impl Default for Config {
    fn default() -> Self {
        Self {
            patterns: vec![],
            invert: false,
            count: false,
            quiet: false,
            word: false,
            line: false,
            context: Context::default(),
            color: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    MatchFound = 0,
    NoMatch = 1,
}

fn build_regex(cfg: &Config) -> Result<Regex, regex::Error> {
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

    RegexBuilder::new(&pat)
        .multi_line(true) // anchors work per-line
        .case_insensitive(false)
        .build()
}

fn highlight_segments(line: &str, re: &Regex) -> String {
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

fn read_to_lines<R: Read>(reader: R) -> io::Result<Vec<String>> {
    let buf = BufReader::new(reader);
    buf.lines().collect()
}

fn open_input(path: Option<&str>) -> io::Result<Box<dyn Read>> {
    match path {
        Some(p) if p != "-" => Ok(Box::new(File::open(p)?)),
        _ => Ok(Box::new(io::stdin())),
    }
}

pub struct RunResult {
    pub output: String,
    pub status: ExitStatus,
}

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
            let printed = if cfg.color && !cfg.line { // even if -x, we'll highlight entire line when it matches; but to be precise, highlight matches
                let hl = highlight_segments(raw_line, &re);
                append_formatted_line(&mut out, name, idx, &hl, true, cfg.line);
                true
            } else {
                append_formatted_line(&mut out, name, idx, raw_line, true, cfg.line);
                true
            };
            let _ = printed;

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
            // Do nothing here; real grep uses -- to separate non-contiguous groups. We'll omit separators for simplicity.
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

fn append_formatted_line(out: &mut String, _filename: Option<&str>, idx: usize, line: &str, _is_match: bool, _line_mode: bool) {
    // Always prefix with 1-based line number, not filename
    let line_no = idx + 1;
    let _ = writeln!(out, "{}:{}", line_no, line);
}

pub fn run(cfg: &Config, inputs: &[String]) -> Result<RunResult, String> {
    if inputs.is_empty() {
        let reader = io::stdin();
        run_on_reader(cfg, reader, None)
    } else if inputs.len() == 1 {
        let name = inputs[0].as_str();
        let reader = open_input(Some(name)).map_err(|e| e.to_string())?;
        run_on_reader(cfg, reader, Some(name))
    } else {
        // Multiple files; aggregate outputs and track status
        let mut out = String::new();
        let mut matched_any = false;
        for name in inputs {
            let rdr = open_input(Some(name)).map_err(|e| e.to_string())?;
            let res = run_on_reader(cfg, rdr, Some(name))?;
            if !cfg.quiet {
                out.push_str(&res.output);
            }
            if res.status == ExitStatus::MatchFound {
                matched_any = true;
            }
        }
        let status = if matched_any { ExitStatus::MatchFound } else { ExitStatus::NoMatch };
        Ok(RunResult { output: if cfg.quiet { String::new() } else { out }, status })
    }
}

// -----------------------
// Tests
// -----------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(patterns: &[&str]) -> Config {
        Config { patterns: patterns.iter().map(|s| s.to_string()).collect(), ..Default::default() }
    }

    #[test]
    fn basic_match() {
        let data = "hello\nworld\nhello world\n";
        let res = run_on_reader(&cfg(&["hello"]), data.as_bytes(), None).unwrap();
        assert_eq!(res.status as i32, ExitStatus::MatchFound as i32);
        assert!(res.output.contains("hello"));
        assert!(!res.output.contains("world\nworld"));
    }

    #[test]
    fn invert_match() {
        let mut c = cfg(&["hello"]);
        c.invert = true;
        let data = "hello\nworld\n";
        let res = run_on_reader(&c, data.as_bytes(), None).unwrap();
        assert!(res.output.contains("world"));
        assert!(!res.output.contains("hello"));
    }

    #[test]
    fn count_only() {
        let mut c = cfg(&["o"]);
        c.count = true;
        let data = "one\ntwo\nthree\n";
        let res = run_on_reader(&c, data.as_bytes(), Some("file.txt")).unwrap();
        assert!(res.output.contains("file.txt:2") || res.output.trim() == "2");
    }

    #[test]
    fn quiet_mode() {
        let mut c = cfg(&["world"]);
        c.quiet = true;
        let data = "hello\nworld\n";
        let res = run_on_reader(&c, data.as_bytes(), None).unwrap();
        assert_eq!(res.output, "");
        assert_eq!(res.status as i32, ExitStatus::MatchFound as i32);
    }

    #[test]
    fn multiple_patterns() {
        let c = cfg(&["foo", "bar"]);
        let data = "x\nbar\ny\n";
        let res = run_on_reader(&c, data.as_bytes(), None).unwrap();
        assert!(res.output.contains("bar"));
    }

    #[test]
    fn word_match() {
        let mut c = cfg(&["he"]);
        c.word = true;
        let data = "he helo she\n";
        let res = run_on_reader(&c, data.as_bytes(), None).unwrap();
        let lines: Vec<_> = res.output.lines().collect();
        // only first token and standalone 'he' should match, not 'helo'
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn line_match() {
        let mut c = cfg(&["hello world"]);
        c.line = true;
        let data = "hello\nhello world\nworld\n";
        let res = run_on_reader(&c, data.as_bytes(), None).unwrap();
        let lines: Vec<_> = res.output.lines().collect();
        assert_eq!(lines, vec!["2:hello world"]);
    }

    #[test]
    fn context_before_after() {
        let mut c = cfg(&["b"]);
        c.context.before = 1;
        c.context.after = 1;
        c.color = false;
        let data = "a\nb\nc\n";
        let res = run_on_reader(&c, data.as_bytes(), None).unwrap();
        let out = res.output;
        // Expect a, b, c each on their own lines with line numbers
        let lines: Vec<_> = out.lines().collect();
        assert_eq!(lines, vec!["1:a", "2:b", "3:c"]);
    }
}
