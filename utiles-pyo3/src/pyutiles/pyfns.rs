use crate::pyutiles::{pycoords, pyparsing};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::{pyfunction, PyErr, PyResult};
use std::collections::HashMap;
use utiles::zoom::ZoomOrZooms;

use crate::pyutiles::pybbox::PyBbox;
use crate::pyutiles::pylnglat::PyLngLat;
use crate::pyutiles::pylnglatbbox::PyLngLatBbox;
use crate::pyutiles::pyparsing::parse_tile_arg;
use crate::pyutiles::pytile::PyTile;
use crate::pyutiles::pytile_tuple::TileTuple;
use crate::pyutiles::pytilelike::PyTileLike;
use crate::pyutiles::pytiles_generator::TilesGenerator;
use crate::pyutiles::zoom::PyZoomOrZooms;

#[pyfunction]
pub fn xyz(x: u32, y: u32, z: u8) -> PyTile {
    PyTile::new(x, y, z)
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn ul(args: &Bound<'_, PyTuple>) -> PyResult<PyLngLat> {
    let tile = parse_tile_arg(args)?;
    let lnglat = tile.ul();
    Ok(lnglat)
}

#[pyfunction]
#[pyo3(signature = (lng, lat, truncate = None))]
pub fn xy(lng: f64, lat: f64, truncate: Option<bool>) -> (f64, f64) {
    utiles::xy(lng, lat, truncate)
}

#[pyfunction]
#[pyo3(signature = (lng, lat, truncate = None))]
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
#[pyo3(signature = (x, y, truncate = None))]
pub fn lnglat(x: f64, y: f64, truncate: Option<bool>) -> PyLngLat {
    // let trunc = truncate.unwrap_or(false);
    let lnglat = utiles::lnglat(x, y, truncate);
    PyLngLat::new(lnglat.lng(), lnglat.lat())
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn bounds(args: &Bound<'_, PyTuple>) -> PyResult<PyLngLatBbox> {
    let tile = parse_tile_arg(args)?;
    let bbox = tile.bounds();
    Ok(bbox)
}

#[pyfunction]
pub fn minmax(zoom: i32) -> PyResult<(u32, u32)> {
    if !(0..=32).contains(&zoom) {
        Err(PyErr::new::<PyValueError, _>(format!(
            "zoom must be between 0 and 32: {zoom}"
        )))?;
    }
    let r = utiles::minmax(zoom as u8);
    Ok(r)
}

#[pyfunction]
pub fn xyz2quadkey(x: u32, y: u32, z: u8) -> String {
    utiles::xyz2quadkey(x, y, z)
}

#[pyfunction]
pub fn quadkey2xyz(quadkey: &str) -> PyResult<PyTile> {
    let xyz = utiles::quadkey2tile(quadkey);
    match xyz {
        Ok(xyz) => Ok(PyTile::from(xyz)),
        Err(e) => Err(PyErr::new::<PyValueError, _>(format!("Error: {e}"))),
    }
}

#[pyfunction]
pub fn qk2xyz(quadkey: &str) -> PyResult<PyTile> {
    quadkey2xyz(quadkey)
}

#[pyfunction]
pub fn from_tuple(tile: TileTuple) -> PyTile {
    PyTile::new(tile.0, tile.1, tile.2)
}
#[pyfunction]
#[pyo3(signature = (tile, fid = None, props = None, projected = None, buffer = None, precision = None))]
pub fn feature(
    py: Python,
    tile: PyTileLike,
    // (u32, u32, u8),
    fid: Option<String>,
    props: Option<HashMap<String, Bound<PyAny>>>,
    projected: Option<String>,
    buffer: Option<f64>,
    precision: Option<i32>,
) -> PyResult<HashMap<String, PyObject>> {
    // Convert the arguments to Rust values
    let pytile: PyTile = tile.into();
    let f = pytile.feature(py, fid, props, projected, buffer, precision)?;
    Ok(f)
}

/// Extract a tile or tiles to Vec<PyTile>
///
/// Consolidated logic from `parse_tiles()` per rec by `@nyurikS` in PR #38
pub fn _extract(arg: &Bound<'_, PyAny>) -> PyResult<Vec<PyTile>> {
    // TODO: this code is identical to parse_tiles() and should be consolidated
    if let Ok(tiles) = arg.extract::<PyTile>() {
        return Ok(vec![tiles]);
    } else if let Ok(tiles) = arg.extract::<Vec<PyTile>>() {
        return Ok(tiles);
    } else if let Ok(seq) = arg.extract::<Vec<(u32, u32, u32)>>() {
        return Ok(seq
            .iter()
            .map(|xyz| PyTile::new(xyz.0, xyz.1, xyz.2 as u8))
            .collect());
    } else if let Ok(seq) = arg.extract::<Vec<Vec<u32>>>() {
        return Ok(seq
            .iter()
            .map(|xyz| PyTile::new(xyz[0], xyz[1], xyz[2] as u8))
            .collect());
    }
    Err(PyErr::new::<PyValueError, _>(
        "the tile argument may have 1 or 4 values. Note that zoom is a keyword-only argument"
    ))
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn xy_bounds(args: &Bound<'_, PyTuple>) -> PyResult<PyBbox> {
    let tile = pyparsing::parse_tile_arg(args)?;
    let web_bbox = utiles::xyz2bbox(tile.xyz.x, tile.xyz.y, tile.xyz.z);
    Ok(PyBbox::new(
        web_bbox.left(),
        web_bbox.bottom(),
        web_bbox.right(),
        web_bbox.top(),
    ))
}

#[pyfunction]
#[pyo3(signature = (lng, lat, zoom, truncate=None))]
pub fn tile(lng: f64, lat: f64, zoom: u8, truncate: Option<bool>) -> PyResult<PyTile> {
    if lat <= -90.0 || lat >= 90.0 {
        Err(PyErr::new::<PyValueError, _>(format!(
            "Invalid latitude: {lat}"
        )))?;
    }
    let xyz = utiles::Tile::from_lnglat_zoom(lng, lat, zoom, truncate);
    match xyz {
        Ok(xyz) => Ok(PyTile::from(xyz)),
        Err(e) => Err(PyErr::new::<PyValueError, _>(format!("Error: {e}"))),
    }
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn pmtileid(args: &Bound<'_, PyTuple>) -> PyResult<u64> {
    let tile = pyparsing::parse_tile_arg(args)?;
    Ok(tile.pmtileid())
}

#[pyfunction]
pub fn pmtileid2xyz(pmtileid: u64) -> PyTile {
    let xyz = utiles::Tile::from_pmtileid(pmtileid);
    PyTile::from(xyz)
}

#[pyfunction]
pub fn from_pmtileid(pmtileid: u64) -> PyTile {
    let xyz = utiles::Tile::from_pmtileid(pmtileid);
    PyTile::from(xyz)
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn quadkey(args: &Bound<'_, PyTuple>) -> PyResult<String> {
    let tile = pyparsing::parse_tile_arg(args)?;
    Ok(utiles::xyz2quadkey(tile.xyz.x, tile.xyz.y, tile.xyz.z))
}

#[pyfunction]
pub fn quadkey_to_tile(quadkey: &str) -> PyResult<PyTile> {
    quadkey2xyz(quadkey)
}

#[pyfunction]
#[pyo3(signature = (* args, zoom = None))]
pub fn parent(args: &Bound<'_, PyTuple>, zoom: Option<u8>) -> PyResult<Option<PyTile>> {
    // Parse the tile argument
    let tile = pyparsing::parse_tile_arg(args)?;
    if tile.xyz.z == 0 {
        return Ok(None);
    }

    // If zoom is not provided, set it to tile.z - 1
    let zoom = zoom.unwrap_or(tile.xyz.z - 1);

    // Check that the zoom level is valid
    if zoom >= tile.xyz.z {
        Err(PyErr::new::<PyValueError, _>(format!(
            "zoom level {} is invalid for tile with zoom {}",
            zoom, tile.xyz.z
        )))?;
    }

    // Calculate the parent tile
    let p = utiles::parent(
        tile.xyz.x,
        tile.xyz.y,
        tile.xyz.z,
        Some(tile.xyz.z - zoom - 1),
    );
    Ok(Some(PyTile::from(p)))
}

#[pyfunction]
#[pyo3(signature = (* args, zoom = None))]
pub fn children(args: &Bound<'_, PyTuple>, zoom: Option<u8>) -> PyResult<Vec<PyTile>> {
    let tile = pyparsing::parse_tile_arg(args)?;
    let zoom = zoom.unwrap_or(tile.xyz.z + 1);
    if zoom < tile.xyz.z {
        Err(PyErr::new::<PyValueError, _>(format!(
            "zoom must be greater than or equal to tile zoom: {}",
            tile.xyz.z
        )))?;
    }
    let children = tile.children(Some(zoom));
    Ok(children)
}

#[pyfunction]
#[pyo3(signature = (* args, zoom = None))]
pub fn neighbors(args: &Bound<'_, PyTuple>, zoom: Option<u8>) -> PyResult<Vec<PyTile>> {
    let tile = pyparsing::parse_tile_arg(args)?;
    let zoom = zoom.unwrap_or(tile.xyz.z);
    if zoom < tile.xyz.z {
        Err(PyErr::new::<PyValueError, _>(format!(
            "zoom must be greater than or equal to tile zoom: {}",
            tile.xyz.z
        )))?;
    }
    Ok(tile.neighbors())
}

#[pyfunction]
#[pyo3(signature = (* args, truncate = None))]
pub fn bounding_tile(
    args: &Bound<'_, PyTuple>,
    truncate: Option<bool>,
) -> PyResult<PyTile> {
    let res = pyparsing::parse_bbox(args);
    if res.is_err() {
        return Err(res.err().unwrap());
    }
    let bbox = res.unwrap();
    let res = utiles::bounding_tile(bbox.into(), truncate)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Error: {e}")))?;
    Ok(PyTile::from(res))
}

#[pyfunction]
pub fn truncate_lnglat(lng: f64, lat: f64) -> (f64, f64) {
    let ll = utiles::LngLat::new(lng, lat);
    let truncated = utiles::truncate_lnglat(&ll);
    (truncated.lng(), truncated.lat())
}

#[pyfunction]
#[pyo3(signature = (west, south, east, north, zooms, truncate=None))]
pub fn tiles_count(
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    zooms: PyZoomOrZooms,
    truncate: Option<bool>,
) -> PyResult<u64> {
    let (west, south, east, north) =
        utiles::bbox_truncate(west, south, east, north, truncate);

    utiles::tiles_count((west, south, east, north), ZoomOrZooms::from(zooms))
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("Error: {e}")))
}

#[pyfunction]
#[pyo3(signature = (west, south, east, north, zooms, truncate=None))]
pub fn tiles(
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    zooms: PyZoomOrZooms,
    truncate: Option<bool>,
) -> PyResult<TilesGenerator> {
    let (west, south, east, north) =
        utiles::bbox_truncate(west, south, east, north, truncate);
    let zooms_vec = match zooms {
        PyZoomOrZooms::Zoom(z) => vec![z],
        PyZoomOrZooms::Zooms(zs) => zs,
    };
    let zooms_vec_iter = zooms_vec.clone();
    let ntiles =
        utiles::tiles_count((west, south, east, north), ZoomOrZooms::from(zooms_vec))
            .map_err(|e| PyErr::new::<PyValueError, _>(format!("Error: {e}")))?;
    let xyzs = utiles::tiles(
        (west, south, east, north),
        ZoomOrZooms::from(zooms_vec_iter),
    )
    .map(PyTile::from);
    Ok(TilesGenerator {
        iter: Box::new(xyzs),
        length: ntiles,
    })
}

#[pyfunction]
#[pyo3(signature = (west, south, east, north, zooms, truncate=None))]
pub fn tiles_list(
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    zooms: PyZoomOrZooms,
    truncate: Option<bool>,
) -> Vec<PyTile> {
    let (west, south, east, north) =
        utiles::bbox_truncate(west, south, east, north, truncate);
    utiles::tiles((west, south, east, north), ZoomOrZooms::from(zooms))
        .map(PyTile::from)
        .collect::<Vec<_>>()
}

#[pyfunction]
pub fn geotransform2optzoom(geotransform: (f64, f64, f64, f64, f64, f64)) -> u8 {
    utiles::geotransform2optzoom(geotransform)
}

#[pyfunction]
pub fn geojson_bounds(obj: &Bound<'_, PyAny>) -> PyResult<PyLngLatBbox> {
    let coordsvec = pycoords::coords(obj)?;
    let mut bbox: (f64, f64, f64, f64) = (180.0, 90.0, -180.0, -90.0);

    for (lng, lat) in coordsvec {
        if lat <= -90.0 || lat >= 90.0 {
            Err(PyErr::new::<PyValueError, _>(format!(
                "Invalid latitude: {lat}"
            )))?;
        }
        bbox = (
            bbox.0.min(lng),
            bbox.1.min(lat),
            bbox.2.max(lng),
            bbox.3.max(lat),
        );
    }
    Ok(PyLngLatBbox::new(bbox.0, bbox.1, bbox.2, bbox.3))
}
