use crate::pyutiles::pylnglat::PyLngLat;
use crate::pyutiles::pylnglatbbox::PyLngLatBbox;
use crate::pyutiles::pytile::PyTile;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyTuple;
use pyo3::{pyfunction, PyErr, PyResult};

#[pyfunction]
pub fn xyz(x: u32, y: u32, z: u8) -> PyTile {
    PyTile::new(x, y, z)
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn ul(args: &PyTuple) -> PyResult<PyLngLat> {
    let tile = crate::parse_tile_arg(args)?;
    let lnglat = tile.ul();
    Ok(lnglat)
}

#[pyfunction]
pub fn xy(lng: f64, lat: f64, truncate: Option<bool>) -> (f64, f64) {
    utiles::xy(lng, lat, truncate)
}

#[pyfunction]
pub fn _xy(lng: f64, lat: f64, truncate: Option<bool>) -> PyResult<(f64, f64)> {
    let trunc = truncate.unwrap_or(false);
    if !trunc && (lat <= -90.0 || lat >= 90.0) {
        Err(PyErr::new::<PyValueError, _>(format!(
            "Invalid latitude: {lat}"
        )))?;
    }
    let xy = utiles::_xy(lng, lat, truncate);
    match xy {
        Ok(xy) => Ok(xy),
        Err(_e) => Err(PyErr::new::<PyValueError, _>(format!(
            "Invalid latitude: {lat}"
        )))?,
    }
}

#[pyfunction]
pub fn lnglat(x: f64, y: f64, truncate: Option<bool>) -> PyLngLat {
    // let trunc = truncate.unwrap_or(false);
    let lnglat = utiles::lnglat(x, y, truncate);
    PyLngLat::new(lnglat.lng(), lnglat.lat())
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn bounds(args: &PyTuple) -> PyResult<PyLngLatBbox> {
    let tile = crate::parse_tile_arg(args)?;
    let bbox = tile.bounds();
    Ok(bbox)
}
