use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

pub fn read_to_lines<R: Read>(reader: R) -> io::Result<Vec<String>> {
    let buf = BufReader::new(reader);
    buf.lines().collect()
}

pub fn open_input(path: Option<&str>) -> io::Result<Box<dyn Read>> {
    match path {
        Some(p) if p != "-" => Ok(Box::new(File::open(p)?)),
        _ => Ok(Box::new(io::stdin())),
    }
}
