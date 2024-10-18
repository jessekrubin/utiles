use pyo3::types::PyType;
use pyo3::{pyclass, pyfunction, pymethods, Bound};
use utiles::tile_type;
use utiles::tile_type::TileType;

#[pyclass(name = "TileType", module = "utiles._utiles")]
pub struct PyTileType(TileType);

#[pymethods]
impl PyTileType {
    #[getter]
    fn format(&self) -> String {
        format!("{}", self.0.format)
    }

    #[getter]
    fn encoding(&self) -> String {
        format!("{}", self.0.encoding)
    }

    #[getter]
    fn compression(&self) -> String {
        format!("{}", self.0.encoding)
    }

    #[getter]
    fn headers(&self) -> Vec<(&'static str, &'static str)> {
        self.0.headers_vec()
    }

    #[classmethod]
    fn from_bytes(_cls: &Bound<'_, PyType>, buffer: &[u8]) -> Self {
        PyTileType(tile_type::tiletype(buffer))
    }
}

#[pyfunction]
pub fn tiletype(buffer: &[u8]) -> PyTileType {
    let ttype = tile_type::tiletype(buffer);
    PyTileType(ttype)
}

#[pyfunction]
pub fn tiletype_str(buffer: &[u8]) -> String {
    tile_type::tiletype_str(buffer)
}

#[pyfunction]
pub fn tiletype2headers(tiletype: usize) -> Vec<(&'static str, &'static str)> {
    tile_type::headers(&tile_type::const2enum(tiletype))
}
