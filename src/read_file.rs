// use std::env;
use std::fs;

use pyo3::prelude::*;
#[pyfunction]
pub fn get_file_contents(file_path: String) -> PyResult<String> {
    let contents = fs::read_to_string(&file_path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("{}", e)))?;
    Ok(contents)
}
