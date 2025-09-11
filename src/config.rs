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
    pub recursive: bool, // -r
    pub case_insensitive: bool, // -i
    pub dotall: bool,   // --dotall
    pub follow: bool,   // -f/--follow
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    MatchFound = 0,
    NoMatch = 1,
}

pub struct RunResult {
    pub output: String,
    pub status: ExitStatus,
}
