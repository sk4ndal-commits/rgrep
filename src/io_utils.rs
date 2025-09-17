//! I/O convenience helpers used by the search engine.
//!
//! These functions provide thin wrappers around standard I/O to read line-oriented
//! input and to open either a named file or stdin via the conventional "-" path.

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

/// Read all lines from a reader into a `Vec<String>` (without trailing newlines).
pub fn read_to_lines<R: Read>(reader: R) -> io::Result<Vec<String>> {
    let buf = BufReader::new(reader);
    buf.lines().collect()
}

/// Open a file path for reading or return stdin when `path` is None or Some("-").
///
/// The returned reader is boxed to allow dynamic dispatch across different sources.
pub fn open_input(path: Option<&str>) -> io::Result<Box<dyn Read>> {
    match path {
        Some(p) if p != "-" => Ok(Box::new(File::open(p)?)),
        _ => Ok(Box::new(io::stdin())),
    }
}
