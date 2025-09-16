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
                .short('e')
                .long("regexp")
                .num_args(1)
                .action(ArgAction::Append)
                .help("Pattern to search for (can be used multiple times)"),
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
                .short('r')
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

/// Parse CLI arguments into a `Config` and input file list.
///
/// Returns `Err(String)` with a human-readable message when validation fails
/// (e.g., no `-e/--regexp` patterns provided).
pub fn parse() -> Result<(Config, Vec<String>), String> {
    let matches = build_cli().get_matches();

    let mut cfg = Config::default();

    if let Some(pats) = matches.get_many::<String>("pattern") {
        cfg.patterns = pats.map(|s| s.to_string()).collect();
    }
    if cfg.patterns.is_empty() {
        return Err("rgrep: no pattern provided; use -e PATTERN".into());
    }

    cfg.invert = matches.get_flag("invert");
    cfg.count = matches.get_flag("count");
    cfg.quiet = matches.get_flag("quiet");
    cfg.word = matches.get_flag("word");
    cfg.line = matches.get_flag("line");

    cfg.recursive = matches.get_flag("recursive");
    cfg.case_insensitive = matches.get_flag("ignore-case");
    cfg.dotall = matches.get_flag("dotall");
    cfg.follow = matches.get_flag("follow");

    let mut before = to_usize(&matches, "before");
    let mut after = to_usize(&matches, "after");
    let ctx = to_usize(&matches, "context");
    if ctx > 0 {
        before = ctx;
        after = ctx;
    }
    cfg.context = Context { before, after };

    let inputs: Vec<String> = matches
        .get_many::<String>("files")
        .map(|vals| vals.map(|s| s.to_string()).collect())
        .unwrap_or_else(|| Vec::new());

    Ok((cfg, inputs))
}
