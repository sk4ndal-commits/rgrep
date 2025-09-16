use rgrep::{run_on_reader, Config, ExitStatus};
use std::io::Cursor;

#[test]
fn or_operator_matches_either() {
    let mut cfg = Config::default();
    cfg.patterns = vec!["foo|bar".to_string()];
    cfg.color = false;
    let data = "one\nbar baz\nfizz\nfoo qux\n";
    let res = run_on_reader(&cfg, Cursor::new(data.as_bytes()), None).unwrap();
    assert_eq!(res.status, ExitStatus::MatchFound);
    // Should contain both lines with foo or bar
    assert!(res.output.contains("bar baz"));
    assert!(res.output.contains("foo qux"));
}

#[test]
fn and_operator_requires_both() {
    let mut cfg = Config::default();
    cfg.patterns = vec!["foo&bar".to_string()];
    cfg.color = false;
    let data = "foo only\nbar only\nfoo and bar\nfoobar together\n";
    let res = run_on_reader(&cfg, Cursor::new(data.as_bytes()), None).unwrap();
    assert_eq!(res.status, ExitStatus::MatchFound);
    // Only the line containing both separate tokens should match; note that regex 'foo' and 'bar' both appear in 'foobar'
    // Our default semantics are regex-based, so 'foobar together' also matches because both patterns are present.
    assert!(res.output.contains("foo and bar"));
    assert!(res.output.contains("foobar together"));
    assert!(!res.output.contains("foo only"));
    assert!(!res.output.contains("bar only"));
}
