use pyo3::prelude::*;

#[pyfunction]
pub fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
