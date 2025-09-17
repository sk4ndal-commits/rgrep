//! Boolean pattern expression parser.
//!
//! This module provides parsing and evaluation of Boolean expressions with patterns,
//! supporting '&' (AND), '|' (OR), and parentheses for grouping.
//! 
//! Examples:
//! - `pattern1&pattern2` - both patterns must match
//! - `pattern1|pattern2` - either pattern must match  
//! - `pattern1&(pattern2|pattern3)` - pattern1 AND (pattern2 OR pattern3)

use regex::{Regex, RegexBuilder};
use crate::config::Config;

#[derive(Debug, Clone)]
pub enum BooleanExpr {
    Pattern(String),
    And(Box<BooleanExpr>, Box<BooleanExpr>),
    Or(Box<BooleanExpr>, Box<BooleanExpr>),
}

impl BooleanExpr {
    /// Evaluate this Boolean expression against a line of text
    pub fn matches(&self, line: &str, regexes: &std::collections::HashMap<String, Regex>) -> bool {
        match self {
            BooleanExpr::Pattern(pattern) => {
                if let Some(regex) = regexes.get(pattern) {
                    regex.is_match(line)
                } else {
                    false
                }
            }
            BooleanExpr::And(left, right) => {
                left.matches(line, regexes) && right.matches(line, regexes)
            }
            BooleanExpr::Or(left, right) => {
                left.matches(line, regexes) || right.matches(line, regexes)
            }
        }
    }

    /// Get all unique patterns from this expression
    pub fn get_patterns(&self) -> std::collections::HashSet<String> {
        let mut patterns = std::collections::HashSet::new();
        self.collect_patterns(&mut patterns);
        patterns
    }

    fn collect_patterns(&self, patterns: &mut std::collections::HashSet<String>) {
        match self {
            BooleanExpr::Pattern(pattern) => {
                patterns.insert(pattern.clone());
            }
            BooleanExpr::And(left, right) | BooleanExpr::Or(left, right) => {
                left.collect_patterns(patterns);
                right.collect_patterns(patterns);
            }
        }
    }
}

/// Parse a Boolean pattern expression
pub fn parse_boolean_expression(input: &str) -> Result<BooleanExpr, String> {
    let mut parser = BooleanParser::new(input);
    parser.parse_or_expression()
}

struct BooleanParser {
    input: Vec<char>,
    pos: usize,
}

impl BooleanParser {
    fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current_char();
        self.pos += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_or_expression(&mut self) -> Result<BooleanExpr, String> {
        let mut left = self.parse_and_expression()?;

        while self.current_char() == Some('|') {
            self.advance(); // consume '|'
            let right = self.parse_and_expression()?;
            left = BooleanExpr::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self) -> Result<BooleanExpr, String> {
        let mut left = self.parse_primary_expression()?;

        while self.current_char() == Some('&') {
            self.advance(); // consume '&'
            let right = self.parse_primary_expression()?;
            left = BooleanExpr::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    fn parse_primary_expression(&mut self) -> Result<BooleanExpr, String> {
        self.skip_whitespace();

        if self.current_char() == Some('(') {
            self.advance(); // consume '('
            let expr = self.parse_or_expression()?;
            self.skip_whitespace();
            if self.current_char() != Some(')') {
                return Err("Expected closing parenthesis".to_string());
            }
            self.advance(); // consume ')'
            Ok(expr)
        } else {
            // Parse pattern until we hit an operator or end
            let mut pattern = String::new();
            let mut escaped = false;
            
            while let Some(ch) = self.current_char() {
                if escaped {
                    pattern.push(ch);
                    escaped = false;
                    self.advance();
                    continue;
                }
                
                if ch == '\\' {
                    pattern.push(ch);
                    escaped = true;
                    self.advance();
                    continue;
                }
                
                if ch == '&' || ch == '|' || ch == ')' || ch.is_whitespace() {
                    break;
                }
                
                pattern.push(ch);
                self.advance();
            }

            if pattern.is_empty() {
                return Err("Expected pattern".to_string());
            }

            Ok(BooleanExpr::Pattern(pattern))
        }
    }
}

/// Build regex map for all patterns in a Boolean expression
pub fn build_pattern_regexes(
    expr: &BooleanExpr,
    cfg: &Config,
) -> Result<std::collections::HashMap<String, Regex>, regex::Error> {
    let patterns = expr.get_patterns();
    let mut regexes = std::collections::HashMap::new();

    for pattern in patterns {
        let mut regex_pattern = pattern.clone();
        
        // Apply word/line constraints
        if cfg.word {
            regex_pattern = format!("\\b(?:{})\\b", regex_pattern);
        }
        if cfg.line {
            regex_pattern = format!("^(?:{})$", regex_pattern);
        }

        let mut builder = RegexBuilder::new(&regex_pattern);
        builder.multi_line(true);
        if cfg.case_insensitive {
            builder.case_insensitive(true);
        }
        if cfg.dotall {
            builder.dot_matches_new_line(true);
        }

        regexes.insert(pattern, builder.build()?);
    }

    Ok(regexes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pattern() {
        let expr = parse_boolean_expression("hello").unwrap();
        match expr {
            BooleanExpr::Pattern(p) => assert_eq!(p, "hello"),
            _ => panic!("Expected pattern"),
        }
    }

    #[test]
    fn test_and_expression() {
        let expr = parse_boolean_expression("hello&world").unwrap();
        match expr {
            BooleanExpr::And(left, right) => {
                match (left.as_ref(), right.as_ref()) {
                    (BooleanExpr::Pattern(l), BooleanExpr::Pattern(r)) => {
                        assert_eq!(l, "hello");
                        assert_eq!(r, "world");
                    }
                    _ => panic!("Expected pattern nodes"),
                }
            }
            _ => panic!("Expected AND expression"),
        }
    }

    #[test]
    fn test_or_expression() {
        let expr = parse_boolean_expression("hello|world").unwrap();
        match expr {
            BooleanExpr::Or(left, right) => {
                match (left.as_ref(), right.as_ref()) {
                    (BooleanExpr::Pattern(l), BooleanExpr::Pattern(r)) => {
                        assert_eq!(l, "hello");
                        assert_eq!(r, "world");
                    }
                    _ => panic!("Expected pattern nodes"),
                }
            }
            _ => panic!("Expected OR expression"),
        }
    }

    #[test]
    fn test_parentheses() {
        let expr = parse_boolean_expression("hello&(world|foo)").unwrap();
        match expr {
            BooleanExpr::And(left, right) => {
                match left.as_ref() {
                    BooleanExpr::Pattern(p) => assert_eq!(p, "hello"),
                    _ => panic!("Expected pattern on left"),
                }
                match right.as_ref() {
                    BooleanExpr::Or(or_left, or_right) => {
                        match (or_left.as_ref(), or_right.as_ref()) {
                            (BooleanExpr::Pattern(l), BooleanExpr::Pattern(r)) => {
                                assert_eq!(l, "world");
                                assert_eq!(r, "foo");
                            }
                            _ => panic!("Expected pattern nodes in OR"),
                        }
                    }
                    _ => panic!("Expected OR expression on right"),
                }
            }
            _ => panic!("Expected AND expression"),
        }
    }
}