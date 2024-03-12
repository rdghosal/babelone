use pyo3::exceptions::PyOSError;
use pyo3::PyResult;
use std::{fs, path::Path};

pub fn read_file(path: &Path) -> PyResult<String> {
    if let Ok(bytes) = fs::read(path) {
        let contents = String::from_utf8(bytes);
        if contents.is_ok() {
            return Ok(contents.unwrap());
        }
    }
    Err(PyOSError::new_err(format!(
        "File {:#?} does not exist or is corrupt.",
        path.to_str()
    )))
}
