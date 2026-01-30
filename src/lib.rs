use pyo3::prelude::*;

mod index;

#[pymodule]
fn the_search_thing(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(index::rust_indexer, m)?)?;
    Ok(())
}
