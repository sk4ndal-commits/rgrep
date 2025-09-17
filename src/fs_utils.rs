//! Filesystem helpers for expanding inputs and detecting binary files.
//!
//! These utilities are used by the search and follow engines to determine what
//! to read and how.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Config;

/// Heuristically determine whether a path refers to a binary file.
///
/// Reads up to 4 KiB from the file and returns true if a NUL byte is observed.
/// The special path "-" is treated as stdin and considered non-binary.
pub fn is_binary_path(path: &str) -> bool {
    if path == "-" { return false; }
    let Ok(mut f) = File::open(path) else { return false; };
    let mut buf = [0u8; 4096];
    match f.read(&mut buf) {
        Ok(n) => buf[..n].iter().any(|&b| b == 0),
        Err(_) => false,
    }
}

/// Expand input paths according to `cfg.recursive` and defaulting rules.
///
/// Behavior:
/// - When `inputs` is empty and `cfg.recursive` is false, returns ["-"] to indicate stdin.
/// - When `inputs` is empty and `cfg.recursive` is true, walks the current directory
///   and returns all files.
/// - When `cfg.recursive` is true and any input is a directory, it is recursively expanded
///   to the files it contains; non-directories are passed through.
pub fn expand_inputs(cfg: &Config, inputs: &[String]) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    if inputs.is_empty() {
        if cfg.recursive {
            // Walk current directory
            for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    files.push(entry.path().to_string_lossy().to_string());
                }
            }
        } else {
            files.push("-".to_string()); // stdin
        }
        return files;
    }

    if cfg.recursive {
        for inp in inputs {
            let p = Path::new(inp);
            if p.is_dir() {
                for entry in WalkDir::new(p).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        files.push(entry.path().to_string_lossy().to_string());
                    }
                }
            } else {
                files.push(inp.clone());
            }
        }
    } else {
        files.extend(inputs.iter().cloned());
    }

    files
}
