use rgrep::{Config, ExitStatus, run_on_reader, run};
use std::io::Cursor;
use std::fs;
use tempfile;

fn create_config(pattern: &str) -> Config {
    Config {
        patterns: vec![pattern.to_string()],
        color: false,
        ..Default::default()
    }
}

// ============ COUNT MODE TESTS ============

#[test]
fn test_count_mode_basic() {
    let mut cfg = create_config("match");
    cfg.count = true;
    
    let data = "match\nno\nmatch\nmatch";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.trim().ends_with("3"), "Should count 3 matches");
}

#[test]
fn test_count_mode_no_matches() {
    let mut cfg = create_config("nomatch");
    cfg.count = true;
    
    let data = "line1\nline2\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::NoMatch);
    assert!(result.output.trim().ends_with("0"), "Should count 0 matches");
}

#[test]
fn test_count_mode_with_or_pattern() {
    let mut cfg = create_config("foo|bar");
    cfg.count = true;
    
    let data = "foo\nbar\nbaz\nfoo bar";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.trim().ends_with("3"), "Should count 3 matches");
}

#[test]
fn test_count_mode_with_and_pattern() {
    let mut cfg = create_config("foo&bar");
    cfg.count = true;
    
    let data = "foo\nbar\nfoo bar\nfoo bar baz";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.trim().ends_with("2"), "Should count 2 matches");
}

#[test]
fn test_count_mode_ignores_context() {
    let mut cfg = create_config("match");
    cfg.count = true;
    cfg.context.before = 5;
    cfg.context.after = 5;
    
    let data = "line1\nmatch\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    // Count mode should ignore context and just count matches
    assert!(result.output.trim().ends_with("1"));
}

// ============ QUIET MODE TESTS ============

#[test]
fn test_quiet_mode_match_found() {
    let mut cfg = create_config("match");
    cfg.quiet = true;
    
    let data = "match\nno match\nmatch";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.is_empty(), "Quiet mode should produce no output");
}

#[test]
fn test_quiet_mode_no_match() {
    let mut cfg = create_config("nomatch");
    cfg.quiet = true;
    
    let data = "line1\nline2\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::NoMatch);
    assert!(result.output.is_empty(), "Quiet mode should produce no output");
}

#[test]
fn test_quiet_mode_with_context() {
    let mut cfg = create_config("match");
    cfg.quiet = true;
    cfg.context.before = 2;
    cfg.context.after = 2;
    
    let data = "line1\nline2\nmatch\nline4\nline5";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.is_empty(), "Quiet mode overrides context");
}

// ============ INVERT MODE TESTS ============

#[test]
fn test_invert_mode_basic() {
    let mut cfg = create_config("skip");
    cfg.invert = true;
    
    let data = "keep\nskip\nkeep";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("keep"));
    assert!(!result.output.contains("skip"));
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_invert_mode_all_match() {
    let mut cfg = create_config("pattern");
    cfg.invert = true;
    
    let data = "pattern\npattern\npattern";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::NoMatch);
    assert!(result.output.is_empty());
}

#[test]
fn test_invert_mode_none_match() {
    let mut cfg = create_config("nomatch");
    cfg.invert = true;
    
    let data = "line1\nline2\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 3, "All lines should match when inverted");
}

#[test]
fn test_invert_mode_with_count() {
    let mut cfg = create_config("skip");
    cfg.invert = true;
    cfg.count = true;
    
    let data = "keep\nskip\nkeep\nskip\nkeep";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.trim().ends_with("3"), "Should count 3 non-matching lines");
}

#[test]
fn test_invert_mode_with_and_pattern() {
    let mut cfg = create_config("foo&bar");
    cfg.invert = true;
    
    let data = "foo\nbar\nfoo bar\nbaz";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    // Lines without both foo AND bar
    assert!(result.output.contains("foo"));
    assert!(result.output.contains("bar"));
    assert!(!result.output.contains("foo bar"));
    assert!(result.output.contains("baz"));
}

// ============ STDIN HANDLING TESTS ============

#[test]
fn test_stdin_single_dash() {
    let cfg = create_config("test");
    let data = "test line\nother line";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("test line"));
}

#[test]
fn test_stdin_no_filename_prefix() {
    let cfg = create_config("test");
    let data = "test";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    // When reading from stdin (name=None), output should not have filename prefix
    // Just line number
    assert!(result.output.starts_with("1:"));
}

// ============ MULTI-FILE TESTS ============

#[test]
fn test_multiple_files() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let file1 = root.join("file1.txt");
    let file2 = root.join("file2.txt");
    
    fs::write(&file1, b"match in file1\nno match").unwrap();
    fs::write(&file2, b"no match\nmatch in file2").unwrap();
    
    let cfg = create_config("match");
    let inputs = vec![
        file1.to_string_lossy().to_string(),
        file2.to_string_lossy().to_string(),
    ];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match in file1"));
    assert!(result.output.contains("match in file2"));
}

#[test]
fn test_multiple_files_no_match() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    
    let file1 = root.join("file1.txt");
    let file2 = root.join("file2.txt");
    
    fs::write(&file1, b"nothing here").unwrap();
    fs::write(&file2, b"nothing here either").unwrap();
    
    let cfg = create_config("nomatch");
    let inputs = vec![
        file1.to_string_lossy().to_string(),
        file2.to_string_lossy().to_string(),
    ];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::NoMatch);
}

#[test]
fn test_single_file_with_filename() {
    let td = tempfile::tempdir().unwrap();
    let file = td.path().join("test.txt");
    fs::write(&file, b"match line").unwrap();
    
    let cfg = create_config("match");
    let inputs = vec![file.to_string_lossy().to_string()];
    
    let result = run(&cfg, &inputs).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match line"));
}

// ============ EDGE CASES ============

#[test]
fn test_empty_file() {
    let cfg = create_config("pattern");
    let data = "";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::NoMatch);
    assert!(result.output.is_empty());
}

#[test]
fn test_single_line_file() {
    let cfg = create_config("match");
    let data = "match";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("match"));
}

#[test]
fn test_very_long_line() {
    let cfg = create_config("needle");
    let long_line = "a".repeat(10000) + "needle" + &"b".repeat(10000);
    let result = run_on_reader(&cfg, Cursor::new(&long_line), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("needle"));
}

#[test]
fn test_many_lines() {
    let cfg = create_config("match");
    let mut data = String::new();
    for i in 0..1000 {
        if i % 100 == 0 {
            data.push_str("match\n");
        } else {
            data.push_str("other\n");
        }
    }
    let result = run_on_reader(&cfg, Cursor::new(&data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    let match_count = result.output.lines().count();
    assert_eq!(match_count, 10);
}

#[test]
fn test_special_characters_in_content() {
    let cfg = create_config(r"\$\d+");
    let data = "$100\n€50\n$25.99";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("$100"));
    assert!(result.output.contains("$25.99"));
    assert!(!result.output.contains("€50"));
}

#[test]
fn test_unicode_content() {
    let cfg = create_config("café");
    let data = "café\ncafe\nкафе";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.contains("café"));
    assert!(!result.output.contains("cafe\n"));
}

#[test]
fn test_unicode_in_pattern() {
    let cfg = create_config("日本");
    let data = "日本語\nEnglish\n日本";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2);
}

// ============ COMBINED FLAGS TESTS ============

#[test]
fn test_count_and_quiet_together() {
    let mut cfg = create_config("match");
    cfg.count = true;
    cfg.quiet = true;
    
    let data = "match\nmatch\nmatch";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.is_empty(), "Quiet should override count output");
}

#[test]
fn test_invert_and_quiet_together() {
    let mut cfg = create_config("skip");
    cfg.invert = true;
    cfg.quiet = true;
    
    let data = "keep\nskip\nkeep";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    assert_eq!(result.status, ExitStatus::MatchFound);
    assert!(result.output.is_empty());
}

#[test]
fn test_word_and_case_insensitive_together() {
    let mut cfg = create_config("WORD");
    cfg.word = true;
    cfg.case_insensitive = true;
    
    let data = "word\nWORD\npassword\nword test";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 3, "Should match whole words case-insensitively");
    assert!(!result.output.contains("password"));
}

#[test]
fn test_line_and_case_insensitive_together() {
    let mut cfg = create_config("TEST");
    cfg.line = true;
    cfg.case_insensitive = true;
    
    let data = "test\nTEST\ntest line\nTest";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 3, "Should match exact lines case-insensitively");
}
