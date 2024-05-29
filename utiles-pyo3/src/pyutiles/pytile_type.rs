use pyo3::pyfunction;
use utiles::tile_type;

#[pyfunction]
pub fn tiletype(buffer: &[u8]) -> usize {
    let ttype = tile_type::tiletype(buffer);
    tile_type::enum2const(&ttype)
}

#[pyfunction]
pub fn tiletype_str(buffer: &[u8]) -> String {
    tile_type::tiletype_str(buffer)
}

#[pyfunction]
pub fn tiletype2headers(tiletype: usize) -> Vec<(&'static str, &'static str)> {
    tile_type::headers(&tile_type::const2enum(tiletype))
}
