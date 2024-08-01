use pyo3::types::PyType;
use pyo3::{pyclass, pyfunction, pymethods, Bound};
use utiles::tile_type;
use utiles::tile_type::TileTypeV2;

#[pyclass(name = "TileType")]
pub struct PyTileType(TileTypeV2);

#[pymethods]
impl PyTileType {
    // #[new]
    // fn new(format: usize, encoding: usize, compression: usize) -> Self {
    //     PyTileType(TileTypeV2 {
    //         format: format as u8,
    //         encoding: encoding as u8,
    //         compression: compression as u8,
    //     })
    // }

    #[getter]
    fn format(&self) -> usize {
        self.0.format as usize
    }

    #[getter]
    fn encoding(&self) -> usize {
        self.0.encoding as usize
    }

    #[getter]
    fn compression(&self) -> usize {
        self.0.encoding as usize
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
