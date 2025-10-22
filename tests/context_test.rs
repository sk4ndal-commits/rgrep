use rgrep::{Config, Context, run_on_reader};
use std::io::Cursor;

fn create_config_with_context(pattern: &str, before: usize, after: usize) -> Config {
    Config {
        patterns: vec![pattern.to_string()],
        context: Context { before, after },
        color: false,
        ..Default::default()
    }
}

#[test]
fn test_no_context() {
    let cfg = create_config_with_context("match", 0, 0);
    let data = "line1\nmatch\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 1, "Should only show matching line");
    assert!(result.output.contains("match"));
}

#[test]
fn test_before_context_one_line() {
    let cfg = create_config_with_context("match", 1, 0);
    let data = "line1\nmatch\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2, "Should show 1 line before + match");
    assert!(result.output.contains("line1"));
    assert!(result.output.contains("match"));
    assert!(!result.output.contains("line3"));
}

#[test]
fn test_after_context_one_line() {
    let cfg = create_config_with_context("match", 0, 1);
    let data = "line1\nmatch\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2, "Should show match + 1 line after");
    assert!(!result.output.contains("line1"));
    assert!(result.output.contains("match"));
    assert!(result.output.contains("line3"));
}

#[test]
fn test_both_before_and_after_context() {
    let cfg = create_config_with_context("match", 1, 1);
    let data = "line1\nmatch\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 3, "Should show before + match + after");
    assert!(result.output.contains("line1"));
    assert!(result.output.contains("match"));
    assert!(result.output.contains("line3"));
}

#[test]
fn test_before_context_multiple_lines() {
    let cfg = create_config_with_context("match", 3, 0);
    let data = "line1\nline2\nline3\nline4\nmatch\nline6";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 4, "Should show 3 lines before + match");
    assert!(result.output.contains("line2"));
    assert!(result.output.contains("line3"));
    assert!(result.output.contains("line4"));
    assert!(result.output.contains("match"));
    assert!(!result.output.contains("line1"));
    assert!(!result.output.contains("line6"));
}

#[test]
fn test_after_context_multiple_lines() {
    let cfg = create_config_with_context("match", 0, 3);
    let data = "line1\nmatch\nline3\nline4\nline5\nline6";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 4, "Should show match + 3 lines after");
    assert!(!result.output.contains("line1"));
    assert!(result.output.contains("match"));
    assert!(result.output.contains("line3"));
    assert!(result.output.contains("line4"));
    assert!(result.output.contains("line5"));
    assert!(!result.output.contains("line6"));
}

#[test]
fn test_before_context_at_start_of_file() {
    let cfg = create_config_with_context("match", 5, 0);
    let data = "match\nline2\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 1, "Should only show match (no lines before it)");
    assert!(result.output.contains("match"));
}

#[test]
fn test_after_context_at_end_of_file() {
    let cfg = create_config_with_context("match", 0, 5);
    let data = "line1\nline2\nmatch";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 1, "Should only show match (no lines after it)");
    assert!(result.output.contains("match"));
}

#[test]
fn test_multiple_matches_with_context() {
    let cfg = create_config_with_context("match", 1, 1);
    let data = "line1\nmatch\nline3\nline4\nmatch\nline6";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    // Should show context around both matches
    assert!(result.output.contains("line1"));
    assert!(result.output.contains("line3"));
    assert!(result.output.contains("line4"));
    assert!(result.output.contains("line6"));
}

#[test]
fn test_overlapping_context_regions() {
    let cfg = create_config_with_context("match", 2, 2);
    let data = "line1\nmatch\nline3\nmatch\nline5";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    // Context regions overlap - line3 is after-context of first match and before-context of second
    let lines: Vec<&str> = result.output.lines().collect();
    assert!(lines.len() >= 4, "Should show both matches with their contexts");
}

#[test]
fn test_context_with_invert_match() {
    let mut cfg = create_config_with_context("nomatch", 1, 1);
    cfg.invert = true;
    
    let data = "line1\nnomatch\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    // With invert, "nomatch" line is NOT matched, so line1 and line3 are matches
    assert!(result.output.contains("line1"));
    assert!(result.output.contains("line3"));
}

#[test]
fn test_large_before_context() {
    let cfg = create_config_with_context("match", 100, 0);
    let data = "line1\nline2\nmatch";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 3, "Should show all available lines before match");
}

#[test]
fn test_large_after_context() {
    let cfg = create_config_with_context("match", 0, 100);
    let data = "match\nline2\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 3, "Should show all available lines after match");
}

#[test]
fn test_context_with_and_pattern() {
    let cfg = create_config_with_context("foo&bar", 1, 1);
    let data = "line1\nfoo bar\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 3, "Should show context around AND match");
}

#[test]
fn test_context_with_or_pattern() {
    let cfg = create_config_with_context("foo|bar", 1, 1);
    let data = "line1\nfoo\nline3\nbar\nline5";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    // Should show context around both matches
    assert!(result.output.contains("line1"));
    assert!(result.output.contains("foo"));
    assert!(result.output.contains("line3"));
    assert!(result.output.contains("bar"));
    assert!(result.output.contains("line5"));
}

#[test]
fn test_context_preserves_line_numbers() {
    let cfg = create_config_with_context("match", 1, 1);
    let data = "line1\nmatch\nline3";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    // Line numbers should be 1-based and correct
    assert!(result.output.contains("1:"));
    assert!(result.output.contains("2:"));
    assert!(result.output.contains("3:"));
}

#[test]
fn test_consecutive_matches_no_context() {
    let cfg = create_config_with_context("match", 0, 0);
    let data = "match\nmatch\nmatch";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 3, "Should show all three matches");
}

#[test]
fn test_consecutive_matches_with_context() {
    let cfg = create_config_with_context("match", 1, 1);
    let data = "line0\nmatch\nmatch\nmatch\nline4";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();
    
    // All matches and their context
    assert!(result.output.contains("line0"));
    assert!(result.output.contains("line4"));
}
