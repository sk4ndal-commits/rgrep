use std::fs::File;
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Config;

pub fn is_binary_path(path: &str) -> bool {
    if path == "-" { return false; }
    let Ok(mut f) = File::open(path) else { return false; };
    let mut buf = [0u8; 4096];
    match f.read(&mut buf) {
        Ok(n) => buf[..n].iter().any(|&b| b == 0),
        Err(_) => false,
    }
}

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
