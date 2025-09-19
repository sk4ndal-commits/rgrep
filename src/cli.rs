//! Command-line argument parsing for the rgrep binary.
//!
//! This module defines the CLI interface (flags and options) and provides a simple
//! `parse()` helper that returns a populated `Config` along with the input paths.
//! On error (e.g., no pattern provided), `parse()` returns a user-friendly message
//! suitable for printing to stderr.

use clap::{Arg, ArgAction, ArgMatches, Command};
use rgrep::{Config, Context};

/// Build the clap Command describing rgrep's CLI.
///
/// This is separated for testability and to support `--help`/`--version` handling
/// by clap. Most consumers should call `parse()` instead.
pub fn build_cli() -> Command {
    Command::new("rgrep")
        .about("A powerful, feature-rich Rust grep implementation")
        .arg(
            Arg::new("pattern")
                .short('r')
                .long("regexp")
                .num_args(1)
                .action(ArgAction::Set)
                .help("Pattern expression to search for (use '|' for OR and '&' for AND; only a single -e is allowed)"),
        )
        .arg(
            Arg::new("word")
                .short('w')
                .long("word-regexp")
                .action(ArgAction::SetTrue)
                .help("Select only those lines containing matches that form whole words"),
        )
        .arg(
            Arg::new("line")
                .short('x')
                .long("line-regexp")
                .action(ArgAction::SetTrue)
                .help("Select only those matches that exactly match the whole line"),
        )
        .arg(
            Arg::new("invert")
                .short('v')
                .long("invert-match")
                .action(ArgAction::SetTrue)
                .help("Invert the sense of matching, to select non-matching lines"),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .action(ArgAction::SetTrue)
                .help("Suppress normal output; instead print a count of matching lines for each input file"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .alias("silent")
                .action(ArgAction::SetTrue)
                .help("Suppress all normal output; only exit status is used"),
        )
        .arg(
            Arg::new("after")
                .short('A')
                .value_name("NUM")
                .num_args(1)
                .help("Print NUM lines of trailing context after matching lines"),
        )
        .arg(
            Arg::new("before")
                .short('B')
                .value_name("NUM")
                .num_args(1)
                .help("Print NUM lines of leading context before matching lines"),
        )
        .arg(
            Arg::new("context")
                .short('C')
                .value_name("NUM")
                .num_args(1)
                .help("Print NUM lines of output context"),
        )
        .arg(
            Arg::new("recursive")
                .short('R')
                .long("recursive")
                .action(ArgAction::SetTrue)
                .help("Read all files under each directory, recursively"),
        )
        .arg(
            Arg::new("ignore-case")
                .short('i')
                .long("ignore-case")
                .action(ArgAction::SetTrue)
                .help("Ignore case distinctions in patterns and data"),
        )
        .arg(
            Arg::new("dotall")
                .long("dotall")
                .action(ArgAction::SetTrue)
                .help("Make '.' match newlines as well (regex dotall mode)"),
        )
        .arg(
            Arg::new("follow")
                .short('f')
                .long("follow")
                .action(ArgAction::SetTrue)
                .help("Follow file(s) for new lines (like tail -f | grep). Only supported for a single file."),
        )
        .arg(
            Arg::new("files")
                .num_args(0..)
                .value_name("FILE")
                .help("Input file(s). Use - for stdin"),
        )
}

/// Parse an optional numeric argument into usize; returns 0 when absent or invalid.
fn to_usize(matches: &ArgMatches, name: &str) -> usize {
    matches
        .get_one::<String>(name)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0)
}

/// Parse a list of optional string arguments into a Vec<String>.
fn get_inputs(matches: &ArgMatches, name: &str) -> Vec<String> {
    matches
        .get_many::<String>(name)
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_else(|| Vec::new())
}

/// Set flags from the parsed `ArgMatches`.
fn set_flags(matches: &ArgMatches, cfg: &mut Config) {
    cfg.invert = matches.get_flag("invert");
    cfg.count = matches.get_flag("count");
    cfg.quiet = matches.get_flag("quiet");
    cfg.word = matches.get_flag("word");
    cfg.line = matches.get_flag("line");

    cfg.recursive = matches.get_flag("recursive");
    cfg.case_insensitive = matches.get_flag("ignore-case");
    cfg.dotall = matches.get_flag("dotall");
    cfg.follow = matches.get_flag("follow");
}

/// Set context from the parsed `ArgMatches`.
fn set_context(matches: &ArgMatches, cfg: &mut Config) {
    let mut before = to_usize(&matches, "before");
    let mut after = to_usize(&matches, "after");
    let ctx = to_usize(&matches, "context");
    if ctx > 0 {
        before = ctx;
        after = ctx;
    }
    cfg.context = Context { before, after };
}

/// Tries setting the pattern from the cmd args, returns true if a pattern was set else false.
fn try_set_pattern(matches: &ArgMatches, cfg: &mut Config) -> bool {
    if let Some(pattern) = matches.get_one::<String>("pattern") {
        cfg.patterns = vec![pattern.to_string()];
    }

    !cfg.patterns.is_empty()
}


/// Parse CLI arguments into a `Config` and input file list.
///
/// Returns `Err(String)` with a human-readable message when validation fails
/// (e.g., no `-e/--regexp` patterns provided).
pub fn parse() -> Result<(Config, Vec<String>), String> {
    let matches = build_cli().get_matches();

    let mut cfg = Config::default();

    if !try_set_pattern(&matches, &mut cfg) {
        return Err("rgrep: no pattern provided; use -r PATTERN".into());
    }

    set_flags(&matches, &mut cfg);
    set_context(&matches, &mut cfg);

    let inputs: Vec<String> = get_inputs(&matches, "files");

    Ok((cfg, inputs))
}
