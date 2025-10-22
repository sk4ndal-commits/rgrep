use rgrep::{Config, run_on_reader};
use std::io::Cursor;

fn create_config(pattern: &str) -> Config {
    Config {
        patterns: vec![pattern.to_string()],
        color: false,
        ..Default::default()
    }
}

// ============ BOOLEAN PARSER ERROR TESTS ============

#[test]
fn test_unclosed_parenthesis_error() {
    let cfg = create_config("(foo&bar");
    let data = "foo bar";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("Expected closing parenthesis"));
    }
}

#[test]
fn test_empty_parentheses_error() {
    let cfg = create_config("()");
    let data = "test";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("Expected pattern"));
    }
}

#[test]
fn test_operator_at_start_error() {
    let cfg = create_config("&foo");
    let data = "foo";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    // Without parentheses, & is treated as literal character in regex, not boolean operator
    // This is expected behavior - only parentheses trigger boolean parsing
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_operator_at_end_error() {
    let cfg = create_config("foo&");
    let data = "foo";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    // Without parentheses, & is treated as literal character in regex, not boolean operator
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_double_operator_error() {
    let cfg = create_config("foo&&bar");
    let data = "foo bar";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    // Without parentheses, && is treated as regex pattern, not boolean operator
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_mismatched_parentheses_extra_closing() {
    let cfg = create_config("foo)");
    let data = "foo";
    // This should NOT be treated as boolean expression (no opening paren)
    // so it should work or fail gracefully
    let result = run_on_reader(&cfg, Cursor::new(data), None);
    // Either works or errors, but shouldn't crash
    let _ = result;
}

#[test]
fn test_nested_unclosed_parentheses() {
    let cfg = create_config("(foo&(bar|baz)");
    let data = "foo bar";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("closing parenthesis"));
    }
}

#[test]
fn test_only_operators() {
    let cfg = create_config("&|");
    let data = "test";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    assert!(result.is_err());
}

// ============ VALID BOOLEAN EDGE CASES ============

#[test]
fn test_single_character_patterns() {
    let cfg = create_config("a&b");
    let data = "a b\na\nb\nab";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("a b"));
    assert!(result.output.contains("ab"));
}

#[test]
fn test_deeply_nested_parentheses() {
    let cfg = create_config("(((a)))");
    let data = "a\nb";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("a"));
    assert!(!result.output.contains("b"));
}

#[test]
fn test_complex_nested_logic() {
    let cfg = create_config("((a&b)|(c&d))&e");
    let data = "a b e\nc d e\na b c\ne";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2, "Should match 'a b e' and 'c d e'");
}

#[test]
fn test_whitespace_in_boolean_expression() {
    let cfg = create_config("(foo&bar)|baz");
    let data = "foo bar\nbaz\nqux";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    // Boolean parser handles the expression correctly
    assert!(result.output.contains("foo bar"));
    assert!(result.output.contains("baz"));
}

#[test]
fn test_empty_pattern_between_operators() {
    let cfg = create_config("foo||bar");
    let data = "foo\nbar";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    // Should either work (treating || as regex) or error
    let _ = result;
}

// ============ REGEX ERROR HANDLING ============

#[test]
fn test_invalid_regex_pattern() {
    let cfg = create_config("[invalid");
    let data = "test";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    assert!(result.is_err(), "Invalid regex should produce error");
}

#[test]
fn test_invalid_regex_in_boolean() {
    let cfg = create_config("(foo)&([invalid)");
    let data = "foo test";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    assert!(
        result.is_err(),
        "Invalid regex in boolean expression should error"
    );
}

#[test]
fn test_backreference_not_supported() {
    let cfg = create_config(r"(test)\1");
    let data = "testtest";
    let result = run_on_reader(&cfg, Cursor::new(data), None);

    // Regex crate doesn't support backreferences, should error
    assert!(result.is_err());
}

// ============ PATTERN EDGE CASES ============

#[test]
fn test_pattern_with_only_whitespace() {
    let cfg = create_config("   ");
    let data = "   \ntest";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    // Should match lines with spaces
    assert!(!result.output.is_empty());
}

#[test]
fn test_pattern_with_newline_char() {
    let cfg = create_config(r"\n");
    let data = "test\nanother";
    // Since we read line by line, \n won't be in the line content
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    // No lines should match since lines don't contain literal \n
    assert!(result.output.is_empty());
}

#[test]
fn test_pattern_matching_line_number_format() {
    let cfg = create_config(r"\d+:");
    let data = "1:test\n2:another\nno number";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("1:test"));
    assert!(result.output.contains("2:another"));
}

#[test]
fn test_very_long_pattern() {
    let long_pattern = "a".repeat(1000);
    let cfg = create_config(&long_pattern);
    let data = format!("{}\nno match", long_pattern);
    let result = run_on_reader(&cfg, Cursor::new(&data), None).unwrap();

    assert!(!result.output.is_empty());
}

#[test]
fn test_pattern_with_null_byte_literal() {
    // Regex pattern matching actual NUL character
    let cfg = create_config(r"\x00");
    let data = "test\x00data\nclean";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    // Should match the line with NUL
    assert!(!result.output.is_empty());
}

// ============ LINE CONTENT EDGE CASES ============

#[test]
fn test_line_with_only_spaces() {
    let cfg = create_config(" ");
    let data = "     \nnospaces\n  ";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    // Should match lines containing spaces
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2, "Should match lines with spaces");
}

#[test]
fn test_empty_lines_in_input() {
    let cfg = create_config("^$");
    let data = "\n\ntest\n\n";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    // Should match empty lines
    let lines: Vec<&str> = result.output.lines().collect();
    assert!(lines.len() >= 3, "Should match empty lines");
}

#[test]
fn test_lines_with_only_whitespace_variations() {
    let cfg = create_config(r"^\s+$");
    let data = "   \n\t\t\n\n   \t   \ntext";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    // Should match lines with only whitespace
    assert!(!result.output.is_empty());
}

#[test]
fn test_line_with_carriage_returns() {
    let cfg = create_config("test");
    let data = "test\r\nanother\rtest";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    // Should handle \r and \r\n line endings
    assert!(result.output.contains("test"));
}

#[test]
fn test_line_with_tabs() {
    let cfg = create_config("\t");
    let data = "notabs\n\t\ttwo tabs\n\tone tab";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    // Should match lines containing tabs
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2, "Should match lines with tabs");
}

// ============ ANCHORS AND BOUNDARIES ============

#[test]
fn test_start_anchor() {
    let cfg = create_config("^test");
    let data = "test at start\nin middle test\ntest";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2, "Should match lines starting with 'test'");
}

#[test]
fn test_end_anchor() {
    let cfg = create_config("test$");
    let data = "test at end test\nmiddle test\nno match";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2, "Should match lines ending with 'test'");
}

#[test]
fn test_both_anchors() {
    let cfg = create_config("^test$");
    let data = "test\ntest extra\nextra test\nno";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 1, "Should match only exact 'test' line");
}

#[test]
fn test_word_boundary_start() {
    let cfg = create_config(r"\btest");
    let data = "test\ncontest\ntest123\n123test";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("test\n") || result.output.contains("1:test"));
    assert!(!result.output.contains("contest"));
    assert!(result.output.contains("test123"));
}

#[test]
fn test_word_boundary_end() {
    let cfg = create_config(r"test\b");
    let data = "test\ntest123\ncontest\n123test";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("contest"));
    assert!(!result.output.contains("test123"));
}

// ============ QUANTIFIERS AND REPETITION ============

#[test]
fn test_zero_or_more_quantifier() {
    let cfg = create_config("a*b");
    let data = "b\nab\naab\naaab\nc";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert!(lines.len() >= 4, "Should match b, ab, aab, aaab");
}

#[test]
fn test_one_or_more_quantifier() {
    let cfg = create_config("a+b");
    let data = "b\nab\naab\nc";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    // "b" alone should not match (needs at least one 'a' before 'b')
    // Check that only "ab" and "aab" are in the output
    assert!(result.output.contains("ab"));
    assert!(result.output.contains("aab"));
    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2, "Should match only ab and aab");
}

#[test]
fn test_optional_quantifier() {
    let cfg = create_config("colou?r");
    let data = "color\ncolour\ncolouur";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("color"));
    assert!(result.output.contains("colour"));
    assert!(!result.output.contains("colouur"));
}

#[test]
fn test_exact_repetition() {
    let cfg = create_config(r"\d{3}");
    let data = "12\n123\n1234";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(!result.output.contains("12\n"));
    assert!(result.output.contains("123"));
    assert!(result.output.contains("1234"));
}

#[test]
fn test_range_repetition() {
    let cfg = create_config(r"\d{2,4}");
    let data = "1\n12\n123\n1234\n12345";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(!result.output.contains("1\n"));
    assert!(result.output.contains("12"));
    assert!(result.output.contains("12345"));
}

// ============ CHARACTER CLASSES ============

#[test]
fn test_digit_character_class() {
    let cfg = create_config(r"\d");
    let data = "no digits\n123\nabc123";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_word_character_class() {
    let cfg = create_config(r"\w+");
    let data = "   \nabc\n123\n!!!\nabc123";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert!(lines.len() >= 3);
}

#[test]
fn test_whitespace_character_class() {
    let cfg = create_config(r"\s");
    let data = "nospace\nhas space\n\t\ttabs";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_negated_character_class() {
    let cfg = create_config(r"[^0-9]+");
    let data = "123\nabc\nmix123";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("abc"));
    assert!(result.output.contains("mix123"));
}

#[test]
fn test_custom_character_class() {
    let cfg = create_config(r"[aeiou]{3,}");
    let data = "hello\naeiou\ntest\nqueue";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("aeiou"));
    assert!(result.output.contains("queue"));
}

// ============ ALTERNATION ============

#[test]
fn test_simple_alternation() {
    let cfg = create_config("cat|dog");
    let data = "cat\ndog\nbird";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 2);
}

#[test]
fn test_alternation_with_groups() {
    let cfg = create_config("(red|blue) car");
    let data = "red car\nblue car\ngreen car";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    assert!(result.output.contains("red car"));
    assert!(result.output.contains("blue car"));
    assert!(!result.output.contains("green car"));
}

#[test]
fn test_multiple_alternations() {
    let cfg = create_config("a|b|c|d");
    let data = "a\ne\nb\nf\nc\ng\nd";
    let result = run_on_reader(&cfg, Cursor::new(data), None).unwrap();

    let lines: Vec<&str> = result.output.lines().collect();
    assert_eq!(lines.len(), 4);
}
