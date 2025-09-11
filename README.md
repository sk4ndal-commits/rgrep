# rgrep

A simple, fast grep-like tool written in Rust. Familiar flags, clear defaults, and a robust follow mode for live logs.

## Features
- Multiple patterns (-e)
- Whole-word (-w) and whole-line (-x) matching
- Invert matches (-v)
- Context lines before/after (-B, -A, -C)
- Count-only (-c); with a single file it prints only the number
- Quiet mode (-q)
- Recursive search (-r)
- Ignore case (-i) and dotall (--dotall)
- Follow a single file (-f) like `tail -f | grep` (with proper context handling)
- Skips binary files automatically
- Optional colorized matches (enabled by default)

## Install
Prerequisite: Rust toolchain (cargo, rustc)

Build:
- Debug: `cargo build`
- Release: `cargo build --release`

The binary will be at `target/debug/rgrep` or `target/release/rgrep`.

## Quick start
Search a file:
```
rgrep -e "error" ./app.log
```

Read from stdin:
```
cat app.log | rgrep -e "timeout"
```

Recursive search:
```
rgrep -r -e "TODO" ./src
```

Case-insensitive:
```
rgrep -i -e "warning" ./logs/*
```

Count-only:
```
# Single file prints just the number
rgrep -c -e "foo" ./file.txt
# Multiple files show file:count per line
rgrep -c -e "foo" ./a.txt ./b.txt
```

Context around matches:
```
# Two lines before and after (-C 2)
rgrep -C 2 -e "panic" ./server.log
# Only after (-A 3) or before (-B 1)
rgrep -A 3 -e "START" ./session.log
rgrep -B 1 -e "END" ./session.log
```

Follow a growing log:
```
# Supports exactly one regular file and starts at EOF
rgrep -f -C 2 -e "ERROR" ./server.log
```

## Behavior
- Count-only, single file: prints only the number. With multiple files: `path:count`.
- Follow mode:
  - One regular file only (not stdin; not multiple files)
  - Starts at end of file; prints newly appended lines only
  - Context (-A/-B/-C) applies within the current append batch; no cross-batch leakage
  - Matches may be color-highlighted; context lines are plain
  - Resilient to transient I/O issues (e.g., rotation)
- Binary files are skipped.

## Exit codes
- 0 — match found
- 1 — no match
- 2 — error (bad args, I/O, etc.)

## Command-line
Common options (see `rgrep -h` for full help):
- `-e, --regexp PATTERN` — pattern (can be used multiple times)
- `-w, --word-regexp` — whole-word matches
- `-x, --line-regexp` — whole-line matches
- `-v, --invert-match` — select non-matching lines
- `-c, --count` — print count of matching lines
- `-q, --quiet` — suppress normal output
- `-A NUM` — trailing context lines
- `-B NUM` — leading context lines
- `-C NUM` — both before/after context
- `-r, --recursive` — recurse into directories
- `-i, --ignore-case` — ignore case
- `--dotall` — dot matches newlines
- `-f, --follow` — follow one file for new lines
- `FILE ...` — input files; use `-` for stdin

## Development
Run tests:
```
cargo test
```

Typical workflow:
- Make changes
- `cargo build`
- `cargo test`

## License
This project is provided as-is; add a LICENSE file if you intend to distribute under a specific license.
