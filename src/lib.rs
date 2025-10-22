//! rgrep: a simple, fast grep-like library and CLI.
//!
//! This crate provides the core search engine used by the rgrep binary, but it can
//! also be embedded as a library. The public API lets you:
//! - Configure search behavior via Config (patterns, context, case, etc.).
//! - Run searches over readers or files (run_on_reader, run).
//! - Follow a single growing file for new matches (follow).
//!
//! Quick example: search a string buffer
//!
//! ```no_run
//! use rgrep::{Config, run_on_reader, ExitStatus};
//! let mut cfg = Config::default();
//! cfg.patterns = vec!["error".into()];
//! let res = run_on_reader(&cfg, "ok\nerror\n".as_bytes(), None).unwrap();
//! assert_eq!(res.status, ExitStatus::MatchFound);
//! println!("{}", res.output);
//! ```
//!
//! Quick example: search files
//!
//! ```no_run
//! use rgrep::{Config, run};
//! let mut cfg = Config::default();
//! cfg.patterns = vec!["TODO".into()];
//! let result = run(&cfg, &["./src".into()]).unwrap();
//! println!("{}", result.output);
//! ```
//!
//! See README for CLI usage examples.

pub mod boolean_parser;
pub mod config;
pub mod follow;
pub mod fs_utils;
pub mod io_utils;
pub mod output;
pub mod regex_utils;
pub mod search;

pub use config::{Config, Context, ExitStatus, RunResult};
pub use follow::follow;
pub use search::{run, run_on_reader};

// -----------------------
// Tests
// -----------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;

    fn cfg(patterns: &[&str]) -> Config {
        Config {
            patterns: patterns.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn case_insensitive_matching() {
        let mut cfg = Config::default();
        cfg.patterns = vec!["hello".to_string()];
        cfg.case_insensitive = true;
        cfg.color = false; // disable color to assert on raw content
        let data = "HeLLo world\nbye";
        let res = run_on_reader(&cfg, Cursor::new(data.as_bytes()), None).unwrap();
        assert_eq!(res.status, ExitStatus::MatchFound);
        assert!(res.output.contains("HeLLo world"));
    }

    #[test]
    fn recursive_traversal() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        let sub = root.join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(root.join("a.txt"), b"foo\nbar\n").unwrap();
        fs::write(sub.join("b.txt"), b"baz\nmatchme\n").unwrap();

        let mut cfg = Config::default();
        cfg.patterns = vec!["matchme".to_string()];
        cfg.recursive = true;
        cfg.color = false;

        let inputs = vec![root.to_string_lossy().to_string()];
        let res = run(&cfg, &inputs).unwrap();
        assert_eq!(res.status, ExitStatus::MatchFound);
        assert!(res.output.contains("matchme"));
    }

    #[test]
    fn binary_files_are_skipped() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        let bin_path = root.join("bin.dat");
        fs::write(&bin_path, [0u8, 159, 146, 150]).unwrap();
        let txt_path = root.join("t.txt");
        fs::write(&txt_path, b"nothing here").unwrap();

        let mut cfg = Config::default();
        cfg.patterns = vec!["zzzz".to_string()];
        cfg.recursive = true;

        let inputs = vec![root.to_string_lossy().to_string()];
        let res = run(&cfg, &inputs).unwrap();
        assert_eq!(res.status, ExitStatus::NoMatch);
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
        // Create a temp file and run via run() to simulate CLI single-file invocation
        let td = tempfile::tempdir().unwrap();
        let path = td.path().join("file.txt");
        std::fs::write(&path, b"one\ntwo\nthree\n").unwrap();
        let inputs = vec![path.to_string_lossy().to_string()];
        let res = run(&c, &inputs).unwrap();
        assert_eq!(res.output.trim(), "2");
    }

    #[test]
    fn multiple_files_count_shows_names() {
        let mut c = cfg(&["o"]);
        c.count = true;
        c.color = false;
        let td = tempfile::tempdir().unwrap();
        let p1 = td.path().join("a.txt");
        let p2 = td.path().join("b.txt");
        std::fs::write(&p1, b"one\nnone\n").unwrap(); // 1 match line
        std::fs::write(&p2, b"two\nzero\n").unwrap(); // 2 lines with 'o' each line, but counting lines -> 2 matches
        let inputs = vec![
            p1.to_string_lossy().to_string(),
            p2.to_string_lossy().to_string(),
        ];
        let res = run(&c, &inputs).unwrap();
        let out = res.output.trim();
        // Expect two lines with filename prefixes
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("a.txt:"));
        assert!(lines[1].contains("b.txt:"));
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
    fn pattern_expression_or() {
        let c = cfg(&["foo|bar"]);
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

#[cfg(test)]
mod more_tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;

    fn cfg(patterns: &[&str]) -> Config {
        Config {
            patterns: patterns.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn error_on_empty_patterns() {
        let cfg = Config::default();
        let res = run_on_reader(&cfg, Cursor::new(b"hello".as_ref()), None);
        assert!(res.is_err());
        assert!(res.err().unwrap().contains("no pattern"));
    }

    #[test]
    fn regex_compile_error() {
        let mut c = Config::default();
        c.patterns = vec!["(".into()];
        let res = run_on_reader(&c, Cursor::new("data"), None);
        assert!(res.is_err());
    }

    #[test]
    fn word_boundary_with_punctuation() {
        let mut c = cfg(&["he"]);
        c.word = true;
        c.color = false;
        let data = "he, she helo\n";
        let res = run_on_reader(&c, Cursor::new(data), None).unwrap();
        let lines: Vec<_> = res.output.lines().collect();
        // Only the line once because only token 'he,' at start counts (same line but printed once)
        assert_eq!(lines.len(), 1);
        assert!(lines[0].ends_with("he, she helo"));
    }

    #[test]
    fn color_highlighting_included_when_enabled() {
        let mut c = cfg(&["hello"]);
        c.color = true; // default, but be explicit
        let data = "say hello there\n";
        let res = run_on_reader(&c, Cursor::new(data), None).unwrap();
        // Expect ANSI escape sequences in output
        assert!(res.output.contains("\u{1b}["));
    }

    #[test]
    fn quiet_no_match_has_empty_output_and_status() {
        let mut c = cfg(&["zzz"]);
        c.quiet = true;
        let res = run_on_reader(&c, Cursor::new("abc\n"), None).unwrap();
        assert_eq!(res.output, "");
        assert_eq!(res.status, ExitStatus::NoMatch);
    }

    #[test]
    fn count_with_invert_counts_non_matching_lines() {
        let mut c = cfg(&["x"]);
        c.count = true;
        c.invert = true;
        let td = tempfile::tempdir().unwrap();
        let p = td.path().join("n.txt");
        fs::write(&p, b"a\nxb\n\n").unwrap();
        let res = run(&c, &[p.to_string_lossy().to_string()]).unwrap();
        // Lines: "a" (non-match), "xb" (match then inverted -> non-match? actually original match true, invert -> false), "" (non-match)
        // So 2 non-matching lines
        assert_eq!(res.output.trim(), "2");
    }

    #[test]
    fn single_file_line_numbers_present() {
        let c = cfg(&["bar"]);
        let data = "foo\nbar\n";
        let res = run_on_reader(&c, Cursor::new(data), Some("file.txt")).unwrap();
        assert!(res.output.starts_with("2:"));
    }

    #[test]
    fn multi_file_count_includes_file_names() {
        let mut c = cfg(&["a"]);
        c.count = true;
        c.color = false;
        let td = tempfile::tempdir().unwrap();
        let p1 = td.path().join("one.txt");
        let p2 = td.path().join("two.txt");
        fs::write(&p1, b"a\n\n").unwrap(); // 1 match line
        fs::write(&p2, b"ba\nca\n").unwrap(); // 2 match lines
        let res = run(
            &c,
            &[
                p1.to_string_lossy().to_string(),
                p2.to_string_lossy().to_string(),
            ],
        )
        .unwrap();
        let mut lines: Vec<_> = res.output.lines().collect();
        lines.sort();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("one.txt:"));
        assert!(lines[1].contains("two.txt:"));
    }

    #[test]
    fn non_existent_file_returns_error() {
        let c = cfg(&["x"]);
        let res = run(&c, &["/definitely/not/exist.txt".to_string()]);
        assert!(res.is_err());
    }

    #[test]
    fn after_context_beyond_end_is_safe() {
        let mut c = cfg(&["last"]);
        c.context.after = 3;
        c.color = false;
        let data = "first\nsecond\nlast\n";
        let res = run_on_reader(&c, Cursor::new(data), None).unwrap();
        let lines: Vec<_> = res.output.lines().collect();
        // Should include last line only, no additional lines beyond EOF
        assert_eq!(lines, vec!["3:last"]);
    }
}
