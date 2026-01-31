// use crate::read_file::get_file_contents;
use pyo3::prelude::*;
use walkdir::WalkDir;

#[pyfunction]
pub fn walk_and_get_files(dir: String) -> PyResult<Vec<String>> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(dir) {
        let entry =
            entry.map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("{}", e)))?;
        paths.push(entry.path().to_string_lossy().to_string());
    }
    Ok(paths)
}
