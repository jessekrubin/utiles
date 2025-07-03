use crate::pyutiles::PyTile;
use geojson::GeoJson;
use pyo3::exceptions::PyValueError;
use pyo3::{PyErr, PyResult, pyfunction};
use utiles::cover::{GeojsonCoverOptions, geojson2tiles as ut_geojson2tiles};

#[pyfunction]
#[pyo3(signature = (geojson_str, maxzoom, minzoom=None))]
pub fn geojson2tiles(
    geojson_str: &str,
    maxzoom: u8,
    minzoom: Option<u8>,
) -> PyResult<Vec<PyTile>> {
    let geojson_res = geojson_str.parse::<GeoJson>();
    match geojson_res {
        Ok(gj) => {
            let options = GeojsonCoverOptions {
                zoom: maxzoom,
                minzoom,
            };
            let tiles = ut_geojson2tiles(&gj, options)
                .map_err(|e| Err(PyErr::new::<PyValueError, _>(format!("Error: {e}"))));
            match tiles {
                Ok(tiles) => {
                    let pytiles = tiles.into_iter().map(PyTile::from).collect();
                    Ok(pytiles)
                }
                Err(e) => e,
            }
        }
        Err(e) => Err(PyErr::new::<PyValueError, _>(format!("Error: {e}"))),
    }
}
