use pyo3::prelude::*;
use mimetype_detector::{detect_file};

#[pyfunction]
pub fn get_file_type(file_path: String) -> PyResult<String> {
    let file_type = detect_file(&file_path)?;
    println!("File Type: {}", file_type);
    Ok(file_type.to_string())
}
