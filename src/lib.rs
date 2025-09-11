pub mod config;
pub mod regex_utils;
pub mod io_utils;
pub mod fs_utils;
pub mod output;
pub mod search;
pub mod follow;

pub use config::{Config, Context, ExitStatus, RunResult};
pub use search::{run_on_reader, run};
pub use follow::follow;

// -----------------------
// Tests
// -----------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::fs;

    fn cfg(patterns: &[&str]) -> Config {
        Config { patterns: patterns.iter().map(|s| s.to_string()).collect(), ..Default::default() }
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
        let inputs = vec![p1.to_string_lossy().to_string(), p2.to_string_lossy().to_string()];
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
