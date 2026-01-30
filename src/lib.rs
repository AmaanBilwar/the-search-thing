use pyo3::prelude::*;

mod aud;
mod helpers;
mod index;
mod read_file;
mod vid;
mod walk;

#[pymodule]
fn the_search_thing(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(index::rust_indexer, m)?)?;
    Ok(())
}
