use std::{error::Error, fs, path::Path};

pub fn read_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(String::from_utf8(fs::read(path)?)?)
}
