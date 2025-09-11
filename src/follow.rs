use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::BufRead;
use std::io::BufReader;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::fs_utils::{expand_inputs, is_binary_path};
use crate::regex_utils::{build_regex, highlight_segments};

#[derive(Debug)]
struct FollowEngine {
    before_n: usize,
    after_n: usize,
    before_buf: VecDeque<String>,
    after_remaining: usize,
}

impl FollowEngine {
    fn new(before_n: usize, after_n: usize) -> Self {
        Self {
            before_n,
            after_n,
            before_buf: VecDeque::with_capacity(before_n.max(1)),
            after_remaining: 0,
        }
    }

    // Process a line and return the lines that should be printed right now,
    // in the right order (before-context lines, the line itself if match, or after-context lines)
    fn handle_line(&mut self, line: String, is_match: bool) -> Vec<String> {
        let mut out = Vec::new();
        if is_match {
            // emit before-context if any
            if self.before_n > 0 {
                for b in &self.before_buf {
                    out.push(b.clone());
                }
            }
            // emit the match line
            out.push(line.clone());
            // set after context counter
            self.after_remaining = self.after_n;
            // reset before buffer (grouping semantics like grep)
            self.before_buf.clear();
        } else if self.after_remaining > 0 {
            // emit line as part of trailing context
            out.push(line.clone());
            self.after_remaining -= 1;
        } else if self.before_n > 0 {
            // keep rolling buffer of leading context candidates
            if self.before_buf.len() == self.before_n {
                self.before_buf.pop_front();
            }
            self.before_buf.push_back(line.clone());
        }
        out
    }
}

pub fn follow(cfg: &Config, inputs: &[String]) -> Result<(), String> {
    validate_follow_inputs(cfg, inputs)?;
    let path = &expand_inputs(cfg, inputs)[0];

    let re = build_regex(cfg).map_err(|e| e.to_string())?;
    let mut pos = get_initial_file_position(path)?;

    follow_file_changes(cfg, path, &re, &mut pos)
}

fn validate_follow_inputs(cfg: &Config, inputs: &[String]) -> Result<(), String> {
    if !cfg.follow {
        return Err("follow mode not enabled".into());
    }

    let files = expand_inputs(cfg, inputs);
    if files.len() != 1 || files[0] == "-" {
        return Err("follow mode supports exactly one regular file".into());
    }

    let path = &files[0];
    if is_binary_path(path) {
        return Err("cannot follow binary file".into());
    }

    Ok(())
}

fn get_initial_file_position(path: &str) -> Result<u64, String> {
    loop {
        match File::open(path) {
            Ok(file) => match file.metadata() {
                Ok(md) => return Ok(md.len()),
                Err(_) => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            },
            Err(_) => {
                // transient error (e.g., file not yet created/rotated)
                thread::sleep(Duration::from_millis(100));
                continue;
            }
        }
    }
}

fn follow_file_changes(
    cfg: &Config,
    path: &str,
    re: &regex::Regex,
    pos: &mut u64,
) -> Result<(), String> {
    let before_n = cfg.context.before;
    let after_n = cfg.context.after;

    loop {
        let meta_len = match fs::metadata(path) {
            Ok(meta) => meta.len(),
            Err(_) => {
                // e.g., file temporarily missing (rotation); wait and retry
                thread::sleep(Duration::from_millis(200));
                continue;
            }
        };

        if meta_len < *pos {
            *pos = meta_len;
        }

        if meta_len > *pos {
            match process_new_file_content(cfg, path, re, pos, before_n, after_n) {
                Ok(new_pos) => *pos = new_pos,
                Err(_) => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn process_new_file_content(
    cfg: &Config,
    path: &str,
    re: &regex::Regex,
    pos: &u64,
    before_n: usize,
    after_n: usize,
) -> Result<u64, std::io::Error> {
    let mut f = File::open(path)?;

    use std::io::Seek;
    f.seek(std::io::SeekFrom::Start(*pos))?;

    let mut reader = BufReader::new(f);
    let mut buf = String::new();
    let mut engine = FollowEngine::new(before_n, after_n);

    loop {
        match reader.read_line(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                let line = buf.trim_end_matches(['\n', '\r']).to_string();
                process_line(cfg, &mut engine, re, line);
                buf.clear();
            }
            Err(e) => return Err(e),
        }
    }

    Ok(fs::metadata(path)?.len())
}

fn process_line(cfg: &Config, engine: &mut FollowEngine, re: &regex::Regex, line: String) {
    let is_match = re.is_match(&line);
    let final_match = if cfg.invert { !is_match } else { is_match };

    let outs = engine.handle_line(line.clone(), final_match);

    if final_match {
        print_match_lines(cfg, re, outs);
    } else {
        print_context_lines(outs);
    }
}

fn print_match_lines(cfg: &Config, re: &regex::Regex, lines: Vec<String>) {
    if cfg.color && !cfg.line {
        let last_idx = lines.len().saturating_sub(1);
        for (i, l) in lines.into_iter().enumerate() {
            if i == last_idx {
                println!("{}", highlight_segments(&l, &re));
            } else {
                println!("{}", l);
            }
        }
    } else {
        for l in lines {
            println!("{}", l);
        }
    }
}

fn print_context_lines(lines: Vec<String>) {
    for l in lines {
        println!("{}", l);
    }
}
