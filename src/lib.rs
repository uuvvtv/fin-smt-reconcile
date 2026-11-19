use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Debug)]
pub struct TextSpan {
    #[pyo3(get, set)]
    pub page: usize,
    #[pyo3(get, set)]
    pub x0: f64,
    #[pyo3(get, set)]
    pub y0: f64,
    #[pyo3(get, set)]
    pub x1: f64,
    #[pyo3(get, set)]
    pub y1: f64,
    #[pyo3(get, set)]
    pub text: String,
}

#[pymethods]
impl TextSpan {
    #[new]
    pub fn new(page: usize, x0: f64, y0: f64, x1: f64, y1: f64, text: String) -> Self {
        Self { page, x0, y0, x1, y1, text }
    }
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TextSpan>()?;
    Ok(())
}
