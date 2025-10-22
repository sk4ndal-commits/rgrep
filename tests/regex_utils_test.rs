use rgrep::{Config, run_on_reader};
use std::io::Cursor;

fn create_config(patterns: Vec<&str>) -> Config {
    Config {
        patterns: patterns.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

#[test]
fn test_simple_or_pattern() {
    let mut cfg = create_config(vec!["foo|bar"]);
    cfg.color = false;

    let data = "foo\nbaz\nbar\nqux";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("foo"));
    assert!(result.output.contains("bar"));
    assert!(!result.output.contains("baz"));
    assert!(!result.output.contains("qux"));
}

#[test]
fn test_simple_and_pattern() {
    let mut cfg = create_config(vec!["foo&bar"]);
    cfg.color = false;

    let data = "foo bar\nfoo\nbar\nfoo bar baz";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "Should match exactly 2 lines with both foo and bar"
    );
}

#[test]
fn test_complex_boolean_with_parentheses() {
    let mut cfg = create_config(vec!["(foo)|(bar&baz)"]);
    cfg.color = false;

    let data = "foo\nbar\nbaz\nbar baz\nfoo bar";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("foo"));
    assert!(result.output.contains("bar baz"));
    assert!(result.output.contains("foo bar"));
}

#[test]
fn test_nested_parentheses() {
    let mut cfg = create_config(vec!["(a&(b|c))"]);
    cfg.color = false;

    let data = "a b\na c\na d\nb c";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2, "Should match 'a b' and 'a c'");
}

#[test]
fn test_word_boundary_flag() {
    let mut cfg = create_config(vec!["test"]);
    cfg.word = true;
    cfg.color = false;

    let data = "test\ntesting\ntest word\ncontest";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("test"));
    assert!(result.output.contains("test word"));
    assert!(!result.output.contains("testing"));
    assert!(!result.output.contains("contest"));
}

#[test]
fn test_line_match_flag() {
    let mut cfg = create_config(vec!["test"]);
    cfg.line = true;
    cfg.color = false;

    let data = "test\ntest word\nword test\ntest";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2, "Should match only exact lines");
}

#[test]
fn test_case_insensitive_with_or() {
    let mut cfg = create_config(vec!["FOO|bar"]);
    cfg.case_insensitive = true;
    cfg.color = false;

    let data = "foo\nBAR\nFOO\nbar\nbaz";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(
        lines.len(),
        4,
        "Should match all case variations of foo or bar"
    );
}

#[test]
fn test_case_insensitive_with_and() {
    let mut cfg = create_config(vec!["HELLO&WORLD"]);
    cfg.case_insensitive = true;
    cfg.color = false;

    let data = "hello world\nHELLO WORLD\nHello World\nhello\nworld";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "Should match all lines with both words regardless of case"
    );
}

#[test]
fn test_dotall_flag() {
    let mut cfg = create_config(vec!["a.b"]);
    cfg.dotall = true;
    cfg.color = false;

    // With dotall, . can match any character including special ones
    // Since we read line-by-line, test that dotall flag is accepted and works within a line
    let data = "axb\na\nb";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("axb"));
}

#[test]
fn test_escaped_special_chars_in_pattern() {
    let mut cfg = create_config(vec![r"test\(value\)"]);
    cfg.color = false;

    let data = "test(value)\ntest value\ntest";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("test(value)"));
    assert!(!result.output.contains("test value"));
}

#[test]
fn test_regex_special_chars() {
    let mut cfg = create_config(vec![r"\d+"]);
    cfg.color = false;

    let data = "abc\n123\ntest456\nno numbers";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("123"));
    assert!(result.output.contains("test456"));
    assert!(!result.output.contains("abc"));
    assert!(!result.output.contains("no numbers"));
}

#[test]
fn test_empty_pattern_error() {
    let cfg = create_config(vec![]);
    let data = "test";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("no pattern"));
    }
}

#[test]
fn test_multiple_and_operators() {
    let mut cfg = create_config(vec!["foo&bar&baz"]);
    cfg.color = false;

    let data = "foo bar baz\nfoo bar\nbar baz\nfoo baz";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "Should match only line with all three terms"
    );
}

#[test]
fn test_multiple_or_operators() {
    let mut cfg = create_config(vec!["foo|bar|baz"]);
    cfg.color = false;

    let data = "foo\nbar\nbaz\nqux";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "Should match lines with any of the three terms"
    );
}

#[test]
fn test_pattern_with_spaces() {
    let mut cfg = create_config(vec!["hello world"]);
    cfg.color = false;

    let data = "hello world\nhello\nworld\nhello  world";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("hello world"));
    assert!(!result.output.lines().any(|l| l.contains("hello\n")));
}

#[test]
fn test_boolean_with_spaces_in_patterns() {
    let mut cfg = create_config(vec!["(hello world)|(foo bar)"]);
    cfg.color = false;

    let data = "hello world\nfoo bar\nhello\nfoo\ntest";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(
        lines.len(),
        2,
        "Should match lines with 'hello world' or 'foo bar'"
    );
}
