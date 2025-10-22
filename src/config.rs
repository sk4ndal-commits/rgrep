/// Controls how many lines of context are shown before and after a match.
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Number of leading lines to include before each matching line.
    pub before: usize,
    /// Number of trailing lines to include after each matching line.
    pub after: usize,
}

/// Configuration for a search run.
///
/// Most fields correspond to familiar grep flags. At minimum, set `patterns` to one or more
/// regular expressions to search for.
#[derive(Debug, Clone)]
pub struct Config {
    /// One or more regex patterns. At least one pattern is required.
    pub patterns: Vec<String>,
    /// Invert the match (like `-v`).
    pub invert: bool, // -v
    /// Print only the count of matching lines (like `-c`).
    pub count: bool, // -c
    /// Suppress normal output; only exit status matters (like `-q`).
    pub quiet: bool, // -q
    /// Match whole words only (like `-w`).
    pub word: bool, // -w
    /// Match the entire line (like `-x`).
    pub line: bool, // -x
    /// Lines of context before/after matches (like `-A`, `-B`, `-C`).
    pub context: Context, // -A, -B, -C
    /// Whether to colorize matches in output (enabled by default).
    pub color: bool, // syntax highlighting
    /// Recurse into directories (like `-r`).
    pub recursive: bool, // -r
    /// Case-insensitive matching (like `-i`).
    pub case_insensitive: bool, // -i
    /// Make `.` match newlines (regex DOTALL).
    pub dotall: bool, // --dotall
    /// Follow a growing single file for new lines (like `-f/--follow`).
    pub follow: bool, // -f/--follow
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
            recursive: false,
            case_insensitive: false,
            dotall: false,
            follow: false,
        }
    }
}

/// Exit status compatible with common grep-style conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// At least one match was found.
    MatchFound = 0,
    /// No matches were found.
    NoMatch = 1,
}

/// Result of a search run.
pub struct RunResult {
    /// Formatted output string (may be empty when `quiet` is set).
    pub output: String,
    /// Status indicating whether any match was found.
    pub status: ExitStatus,
}
