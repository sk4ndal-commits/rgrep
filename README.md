rgrep — A feature-rich grep implemented in Rust

rgrep is a command-line tool for searching text using regular expressions, written in Rust. It aims to be familiar for grep users while providing clear defaults, useful context printing, colorized matches, recursive search, and a robust follow mode for live-updating files.

Highlights
- Multiple patterns with -e
- Word and full-line matches (-w, -x)
- Invert matches (-v)
- Context lines before/after matches (-B, -A, -C)
- Count-only mode (-c) with single-file friendly output (prints just the number)
- Quiet mode (-q)
- Recursive directory search (-r)
- Case-insensitive (-i) and dotall (--) regex options
- Follow mode (-f) to watch a single log file like tail -f | grep, including proper context handling
- Skips binary files automatically
- Colorized output for matches (enabled by default)

Installation
1) Prerequisites: Rust toolchain (cargo and rustc)
2) Build:
   - Debug: cargo build
   - Release: cargo build --release
3) The binary will be at target/debug/rgrep or target/release/rgrep.

Quick start
- Search for a pattern in a file:
  rgrep -e "error" ./app.log

- Read from stdin (default when no files are provided):
  cat app.log | rgrep -e "timeout"

- Recursive search under a directory:
  rgrep -r -e "TODO" ./src

- Case-insensitive search:
  rgrep -i -e "warning" ./logs/*

- Count-only mode:
  # Single file prints just the number
  rgrep -c -e "foo" ./file.txt
  # Multiple files show file:count per line
  rgrep -c -e "foo" ./a.txt ./b.txt

- Context around matches:
  # Two lines before and after (-C 2)
  rgrep -C 2 -e "panic" ./server.log
  # Only lines after (-A 3) or before (-B 1)
  rgrep -A 3 -e "START" ./session.log
  rgrep -B 1 -e "END" ./session.log

- Follow a growing log file (like tail -f | grep):
  # Note: follow mode supports exactly one regular file and starts reading at EOF.
  rgrep -f -C 2 -e "ERROR" ./server.log

Behavior notes
- Count-only single file: When using -c with a single file, rgrep prints only the numeric count, with no file path prefix. With multiple files, it prints file:count per line.
- Follow mode:
  - Only one regular file is supported (not stdin, not multiple files).
  - Starts reading at the end of the file and prints only newly appended lines.
  - Context (-A/-B/-C) applies to lines appended in the current growth batch only; context does not leak across separate appends.
  - Matching lines are color-highlighted (if color is enabled); context lines are printed plainly.
  - rgrep is resilient to transient I/O issues during follow (e.g., log rotation) and will keep retrying.
- Binary files: rgrep skips binary files automatically.

Exit codes
- 0: Match found
- 1: No match
- 2: Error (invalid arguments, I/O errors, etc.)

Command-line reference
- -e, --regexp PATTERN
  Pattern to search for (can be used multiple times).
- -w, --word-regexp
  Select only those lines containing matches that form whole words.
- -x, --line-regexp
  Select only those matches that exactly match the whole line.
- -v, --invert-match
  Invert the sense of matching, to select non-matching lines.
- -c, --count
  Suppress normal output; instead print a count of matching lines.
- -q, --quiet (alias: --silent)
  Suppress all normal output; only exit status is used.
- -A NUM
  Print NUM lines of trailing context after matching lines.
- -B NUM
  Print NUM lines of leading context before matching lines.
- -C NUM
  Print NUM lines of output context (both before and after).
- -r, --recursive
  Read all files under each directory, recursively.
- -i, --ignore-case
  Ignore case distinctions in patterns and data.
- --dotall
  Make '.' match newlines as well (regex dotall mode).
- -f, --follow
  Follow a single file for new lines (like tail -f | grep). Only supported for one regular file.
- FILE ...
  One or more input files. Use - (dash) to read from stdin.

Examples
- Search multiple patterns:
  rgrep -e "foo" -e "bar" ./file.txt

- Whole word and full line matches:
  rgrep -w -e "cat" ./text.txt
  rgrep -x -e "^OK$" ./status.txt

- Invert match to show non-matching lines:
  rgrep -v -e "debug" ./app.log

Development
- Run the test suite:
  cargo test

- Suggested workflow:
  - Make changes
  - cargo build
  - cargo test

License
This project is provided as-is; see repository context or add a LICENSE file if you intend to distribute under a specific license.
