use std::fs::File;
use std::path::Path;
use std::io::{Error, Read};

pub fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut file = File::open(path)?;
    let buf_size = file.metadata().map(|m| m.len() as usize + 1).unwrap_or(0);
    let mut string = String::with_capacity(buf_size);
    file.read_to_string(&mut string)?;
    Ok(string)
}
