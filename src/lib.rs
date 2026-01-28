use pyo3::prelude::*;

mod file_detector_with_extension;
mod file_detector_with_mimetype;
mod read_file;
mod walk;

#[pymodule]
fn the_search_thing(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(walk::walk_and_get_content, m)?)?;
    m.add_function(wrap_pyfunction!(
        file_detector_with_mimetype::get_file_type,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(
        file_detector_with_extension::get_file_type_with_extension,
        m
    )?)?;
    Ok(())
}
