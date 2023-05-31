use std::collections::{HashMap, HashSet};

use crate::utiles::BBox;

use pyo3::exceptions::{self, PyValueError};

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyutiles::pybbox::PyBbox;
use pyutiles::pyiters::CoordinateIterator;
use pyutiles::pylnglat::PyLngLat;
use pyutiles::pylnglatbbox::PyLngLatBbox;
use pyutiles::pytile::PyTile;

use utiles::zoom::ZoomOrZooms;

use utiles::libtiletype;

mod pyutiles;
mod utiles;

#[derive(FromPyObject)]
pub struct TileTuple(u32, u32, u8);

impl From<PyTile> for TileTuple {
    fn from(tile: PyTile) -> Self {
        Self(tile.xyz.x, tile.xyz.y, tile.xyz.z)
    }
}

impl From<Vec<u32>> for TileTuple {
    fn from(tile: Vec<u32>) -> Self {
        Self(tile[0], tile[1], tile[2] as u8)
    }
}

impl From<PyLngLatBbox> for BBox {
    fn from(val: PyLngLatBbox) -> Self {
        val.bbox
    }
}

#[pyfunction]
fn minmax(zoom: i32) -> PyResult<(u32, u32)> {
    if !(0..=32).contains(&zoom) {
        return Err(PyErr::new::<PyValueError, _>(format!(
            "zoom must be between 0 and 32: {zoom}"
        )))?;
    }
    Ok(utiles::minmax(zoom as u32))
}

#[pyfunction]
fn xyz2quadkey(x: u32, y: u32, z: u8) -> String {
    utiles::xyz2quadkey(x, y, z)
}

#[pyfunction]
fn quadkey2xyz(quadkey: &str) -> PyResult<PyTile> {
    let xyz = utiles::quadkey2tile(quadkey);
    match xyz {
        Ok(xyz) => Ok(PyTile::from(xyz)),
        Err(e) => Err(PyErr::new::<PyValueError, _>(format!("Error: {e}"))),
    }
}

#[pyfunction]
fn from_tuple(tile: TileTuple) -> PyTile {
    PyTile::new(tile.0, tile.1, tile.2)
}

#[pyfunction]
fn tiletype(buffer: &[u8]) -> usize {
    libtiletype::enum2const(libtiletype::tiletype(buffer))
}

#[pyfunction]
fn tiletype_str(buffer: &[u8]) -> String {
    libtiletype::tiletype_str(buffer)
}

#[pyfunction]
fn tiletype2headers(tiletype: usize) -> Vec<(&'static str, &'static str)> {
    let headers = libtiletype::headers(libtiletype::const2enum(tiletype));
    headers
}

#[pyfunction]
#[pyo3(signature = (* args))]
fn parse_tile_arg(args: &PyTuple) -> PyResult<PyTile> {
    if args.len() == 1 {
        let arg = args.get_item(0)?;
        if let Ok(tile) = arg.extract::<PyTile>() {
            return Ok(tile);
        } else if let Ok(seq) = arg.extract::<(u32, u32, u8)>() {
            return Ok(PyTile::new(seq.0, seq.1, seq.2));
        } else if let Ok(seq) = arg.extract::<Vec<u32>>() {
            return Ok(PyTile::new(seq[0], seq[1], seq[2] as u8));
        }
    } else if args.len() == 3 {
        let x = args.get_item(0)?.extract()?;
        let y = args.get_item(1)?.extract()?;
        let z = args.get_item(2)?.extract()?;
        return Ok(PyTile::new(x, y, z));
    }

    Err(PyErr::new::<PyValueError, _>(
        "the tile argument may have 1 or 3 values. Note that zoom is a keyword-only argument"
    ))
}

#[pyfunction]
#[pyo3(signature = (* args))]
fn parse_bbox(args: &PyTuple) -> PyResult<PyLngLatBbox> {
    let arglen = args.len();
    match arglen {
        1 => {
            let arg = args.get_item(0)?;
            if let Ok(bbox) = arg.extract::<(f64, f64, f64, f64)>() {
                return Ok(PyLngLatBbox::new(bbox.0, bbox.1, bbox.2, bbox.3));
            } else if let Ok(seq) = arg.extract::<Vec<f64>>() {
                return Ok(PyLngLatBbox::new(seq[0], seq[1], seq[2], seq[3]));
            }
            // raise ValueError("the bbox argument may have 1 or 4 values")
            Err(PyErr::new::<PyValueError, _>(
                "the bbox argument may have 1, 2 or 4 values",
            ))
        }
        2 => {
            let x = args.get_item(0)?.extract()?;
            let y = args.get_item(1)?.extract()?;
            Ok(PyLngLatBbox::new(x, y, x, y))
        }
        4 => {
            let x1 = args.get_item(0)?.extract()?;
            let y1 = args.get_item(1)?.extract()?;
            let x2 = args.get_item(2)?.extract()?;
            let y2 = args.get_item(3)?.extract()?;
            Ok(PyLngLatBbox::new(x1, y1, x2, y2))
        }
        _ => Err(PyErr::new::<PyValueError, _>(
            "the bbox argument may have 1, 2 or 4 values",
        ))?,
    }
}

fn _extract(arg: &PyAny) -> PyResult<Vec<PyTile>> {
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
fn parse_tiles(args: &PyTuple) -> PyResult<Vec<PyTile>> {
    if args.len() == 1 {
        let arg = args.get_item(0)?;
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
    } else if args.len() == 3 {
        // if the first value is a number assume the thing is a tile
        if let Ok(x) = args.get_item(0)?.extract::<u32>() {
            let y = args.get_item(1)?.extract()?;
            let z = args.get_item(2)?.extract()?;
            return Ok(vec![PyTile::new(x, y, z)]);
        }
    }

    Err(PyErr::new::<PyValueError, _>(
        "the tile argument may have 1 or 3 values. Note that zoom is a keyword-only argument"
    ))
}

#[pyfunction]
#[pyo3(signature = (* args))]
fn _parse_tile_arg(args: &PyTuple) -> PyResult<PyTile> {
    parse_tile_arg(args)
}

#[pyfunction]
#[pyo3(signature = (* args))]
fn ul(args: &PyTuple) -> PyResult<PyLngLat> {
    let tile = parse_tile_arg(args)?;
    let lnglat = tile.ul();
    Ok(lnglat)
}

#[pyfunction]
fn xy(lng: f64, lat: f64, truncate: Option<bool>) -> PyResult<(f64, f64)> {
    let xy = utiles::xy(lng, lat, truncate);
    Ok(xy)
}

#[pyfunction]
fn _xy(lng: f64, lat: f64, truncate: Option<bool>) -> PyResult<(f64, f64)> {
    let trunc = truncate.unwrap_or(false);
    if !trunc && (lat <= -90.0 || lat >= 90.0) {
        return Err(PyErr::new::<PyValueError, _>(format!(
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
fn lnglat(x: f64, y: f64, truncate: Option<bool>) -> PyResult<PyLngLat> {
    // let trunc = truncate.unwrap_or(false);
    let lnglat = utiles::lnglat(x, y, truncate);
    Ok(PyLngLat::new(lnglat.lng(), lnglat.lat()))
}

#[pyfunction]
#[pyo3(signature = (* args))]
fn bounds(args: &PyTuple) -> PyResult<PyLngLatBbox> {
    let tile = parse_tile_arg(args)?;
    let bbox = tile.bounds();
    Ok(bbox)
}

#[pyfunction]
#[pyo3(signature = (* args))]
fn xy_bounds(args: &PyTuple) -> PyResult<PyBbox> {
    let tile = parse_tile_arg(args)?;
    let pybbox = utiles::xyz2bbox(tile.xyz.x, tile.xyz.y, tile.xyz.z);
    Ok(PyBbox::new(
        pybbox.left,
        pybbox.bottom,
        pybbox.right,
        pybbox.top,
    ))
}

#[pyfunction]
fn tile(lng: f64, lat: f64, zoom: u8, truncate: Option<bool>) -> PyResult<PyTile> {
    if lat <= -90.0 || lat >= 90.0 {
        return Err(PyErr::new::<PyValueError, _>(format!(
            "Invalid latitude: {lat}"
        )))?;
    }
    let xyz = utiles::Tile::from_lnglat_zoom(lng, lat, zoom, truncate);
    Ok(PyTile::from(xyz))
}

#[pyfunction]
#[pyo3(signature = (* args))]
fn pmtileid(args: &PyTuple) -> PyResult<u64> {
    let tile = parse_tile_arg(args)?;
    Ok(tile.pmtileid())
}

#[pyfunction]
fn from_pmtileid(pmtileid: u64) -> PyResult<PyTile> {
    let xyz = utiles::Tile::from_pmtileid(pmtileid);
    Ok(PyTile::from(xyz))
}

#[pyfunction]
#[pyo3(signature = (* args))]
fn quadkey(args: &PyTuple) -> PyResult<String> {
    let tile = parse_tile_arg(args)?;
    Ok(utiles::xyz2quadkey(tile.xyz.x, tile.xyz.y, tile.xyz.z))
}

#[pyfunction]
fn quadkey_to_tile(quadkey: &str) -> PyResult<PyTile> {
    let res = utiles::quadkey2tile(quadkey);
    let xyz = match res {
        Ok(xyz) => xyz,
        Err(_e) => Err(PyErr::new::<PyValueError, _>(format!(
            "Invalid quadkey: {quadkey}"
        )))?,
    };
    Ok(PyTile::from(xyz))
}

#[pyfunction]
#[pyo3(signature = (* args, zoom = None))]
fn parent(args: &PyTuple, zoom: Option<u8>) -> PyResult<Option<PyTile>> {
    // Parse the tile argument
    let tile = parse_tile_arg(args)?;
    if tile.xyz.z == 0 {
        return Ok(None);
    }

    // If zoom is not provided, set it to tile.z - 1
    let zoom = zoom.unwrap_or(tile.xyz.z - 1);

    // Check that the zoom level is valid
    if zoom >= tile.xyz.z {
        return Err(PyErr::new::<PyValueError, _>(format!(
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
fn children(args: &PyTuple, zoom: Option<u8>) -> PyResult<Vec<PyTile>> {
    let tile = parse_tile_arg(args)?;
    let zoom = zoom.unwrap_or(tile.xyz.z + 1);
    if zoom < tile.xyz.z {
        return Err(PyErr::new::<PyValueError, _>(format!(
            "zoom must be greater than or equal to tile zoom: {}",
            tile.xyz.z
        )))?;
    }
    let children = tile.children(Some(zoom));
    Ok(children)
}

#[pyfunction]
#[pyo3(signature = (* args, zoom = None))]
fn neighbors(args: &PyTuple, zoom: Option<u8>) -> PyResult<Vec<PyTile>> {
    let tile = parse_tile_arg(args)?;
    let zoom = zoom.unwrap_or(tile.xyz.z);
    // if zoom < 0 {
    //     return Err(PyErr::new::<PyValueError, _>(format!(
    //         "zoom must be greater than or equal to tile zoom: {}",
    //         tile.z
    //     )))?;
    // }
    if zoom < tile.xyz.z {
        return Err(PyErr::new::<PyValueError, _>(format!(
            "zoom must be greater than or equal to tile zoom: {}",
            tile.xyz.z
        )))?;
    }
    Ok(tile.neighbors())
}

#[pyfunction]
#[pyo3(signature = (* args, truncate = None))]
fn bounding_tile(args: &PyTuple, truncate: Option<bool>) -> PyResult<PyTile> {
    let res = parse_bbox(args);
    if res.is_err() {
        return Err(res.err().unwrap());
    }
    let bbox = res.unwrap();
    let res = utiles::bounding_tile(bbox.into(), truncate);
    Ok(PyTile::from(res))
}

#[pyfunction]
fn truncate_lnglat(lng: f64, lat: f64) -> PyResult<(f64, f64)> {
    let ll = utiles::LngLat::new(lng, lat);
    let truncated = utiles::truncate_lnglat(&ll);
    Ok((truncated.lng(), truncated.lat()))
}

#[derive(FromPyObject)]
enum PyTileLike {
    #[pyo3(transparent, annotation = "tuple[int, int, int]")]
    Tuple3d((u32, u32, u8)),

    #[pyo3(transparent, annotation = "Tile")]
    PyTile(PyTile),
}

impl From<PyTileLike> for PyTile {
    fn from(val: PyTileLike) -> Self {
        match val {
            PyTileLike::Tuple3d((x, y, z)) => PyTile::from(utiles::Tile::new(x, y, z)),
            PyTileLike::PyTile(t) => t,
        }
    }
}

#[derive(FromPyObject)]
enum PyZoomOrZooms {
    #[pyo3(transparent, annotation = "int")]
    Zoom(u8),
    #[pyo3(transparent, annotation = "list[int]")]
    Zooms(Vec<u8>),
}

impl From<PyZoomOrZooms> for ZoomOrZooms {
    fn from(val: PyZoomOrZooms) -> Self {
        match val {
            PyZoomOrZooms::Zoom(z) => ZoomOrZooms::Zoom(z),
            PyZoomOrZooms::Zooms(zs) => ZoomOrZooms::Zooms(zs),
        }
    }
}

#[pyclass]
struct TilesGenerator {
    iter: Box<dyn Iterator<Item = PyTile> + Send>,
}

#[pymethods]
impl TilesGenerator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyTile> {
        slf.iter.next()
    }
}

#[pyfunction]
fn tiles(
    west: f64,
    south: f64,
    east: f64,
    north: f64,
    zooms: PyZoomOrZooms,
    truncate: Option<bool>,
) -> TilesGenerator {
    let (west, south, east, north) =
        utiles::bbox_truncate(west, south, east, north, truncate);

    let xyzs = utiles::tiles((west, south, east, north), ZoomOrZooms::from(zooms))
        .map(PyTile::from);
    TilesGenerator {
        iter: Box::new(xyzs),
    }
}

#[pyfunction]
fn tiles_list(
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
fn xyz(x: u32, y: u32, z: u8) -> PyTile {
    PyTile::new(x, y, z)
}

#[derive(FromPyObject, Debug)]
enum CoordsExtractor<'a> {
    ListVecF64(Vec<Vec<f64>>),
    VecF64(Vec<f64>),
    IntTuple3d((i32, i32, i32)),
    IntTuple2d((i32, i32)),
    List(Vec<&'a PyAny>),
    Tuple(Vec<&'a PyAny>),
    Dict(HashMap<String, &'a PyAny>),
    #[pyo3(transparent)]
    CatchAll(&'a PyAny), // This extraction never fails
}

#[pyfunction]
fn _coords(_py: Python, obj: &PyAny) -> PyResult<CoordinateIterator> {
    let thing = CoordsExtractor::extract(obj)?;
    match thing {
        CoordsExtractor::ListVecF64(v) => {
            // ensure 2d
            let iter = v.into_iter().map(|t| (t[0], t[1]));
            Ok(CoordinateIterator {
                iter: Box::new(iter.into_iter()),
            })
        }
        CoordsExtractor::VecF64(v) => {
            // ensure 2d
            let vec = vec![(v[0], v[1])];
            Ok(CoordinateIterator {
                iter: Box::new(vec.into_iter()),
            })
        }
        CoordsExtractor::IntTuple3d(t) => {
            let iter = vec![(f64::from(t.0), f64::from(t.1))];
            Ok(CoordinateIterator {
                iter: Box::new(iter.into_iter()),
            })
        }
        CoordsExtractor::IntTuple2d(t) => {
            // return an iterator of the tuple
            let iter = vec![(f64::from(t.0), f64::from(t.1))];
            Ok(CoordinateIterator {
                iter: Box::new(iter.into_iter()),
            })
        }
        CoordsExtractor::List(l) => {
            if l.len() == 2 {
                // try to extract as coords
                return Ok(CoordinateIterator {
                    iter: Box::new(
                        vec![(l[0].extract::<f64>()?, l[1].extract::<f64>()?)]
                            .into_iter(),
                    ),
                });
            }
            let mut coordsvec: Vec<(f64, f64)> = Vec::new();
            for item in &l {
                let c = _coords(_py, item)?;
                let cv = c.iter.collect::<Vec<_>>();
                coordsvec.extend(cv)
            }
            Ok(CoordinateIterator {
                iter: Box::new(coordsvec.into_iter()),
            })
        }
        CoordsExtractor::Tuple(t) => {
            if t.is_empty() {
                return Ok(CoordinateIterator {
                    iter: Box::new(vec![].into_iter()),
                });
            }
            if t.len() == 1 {
                let res = _coords(_py, t[0]);
                return res;
            }
            Ok(CoordinateIterator {
                iter: Box::new(vec![].into_iter()),
            })
        }
        CoordsExtractor::Dict(d) => {
            // extract the sub dict key 'coordinates'
            if let Some(coords) = d.get("coordinates") {
                let res = _coords(_py, coords);
                return res;
            }
            // extract the sub dict
            if let Some(geom) = d.get("geometry") {
                // recurse around again
                let res = _coords(_py, geom);
                return Ok(res.unwrap());
            }
            if let Some(features) = d.get("features") {
                if let Ok(features) = features.extract::<Vec<&PyDict>>() {
                    // chain the iterators
                    let mut coords = vec![];
                    for feature in features {
                        let res = _coords(_py, feature)?;
                        coords.extend(res.iter);
                    }
                    return Ok(CoordinateIterator {
                        iter: Box::new(coords.into_iter()),
                    });
                }
                // return empty iterator
                return Ok(CoordinateIterator {
                    iter: Box::new(vec![].into_iter()),
                });
            }
            // return empty iterator
            Ok(CoordinateIterator {
                iter: Box::new(vec![].into_iter()),
            })
        }
        CoordsExtractor::CatchAll(_c) => {
            Err(PyErr::new::<exceptions::PyTypeError, _>("NO COORDS"))
        }
    }
}

fn merge(merge_set: &HashSet<PyTile>) -> (HashSet<PyTile>, bool) {
    let mut upwards_merge: HashMap<PyTile, HashSet<PyTile>> = HashMap::new();
    for tile in merge_set {
        let tile_parent = tile.parent(None);
        let children_set = upwards_merge
            .entry(tile_parent)
            .or_insert_with(HashSet::new);
        children_set.insert(*tile);
    }
    let mut current_tileset: Vec<PyTile> = Vec::new();
    let mut changed = false;
    for (supertile, children) in upwards_merge {
        if children.len() == 4 {
            current_tileset.push(supertile);
            changed = true;
        } else {
            current_tileset.extend(children);
        }
    }
    (current_tileset.into_iter().collect(), changed)
}

#[pyfunction]
#[pyo3(signature = (* args))]
fn simplify(_py: Python, args: &PyTuple) -> PyResult<HashSet<PyTile>> {
    // Parse tiles from the input sequence
    let tiles = parse_tiles(args)?;
    let mut _tiles = tiles.into_iter().collect::<Vec<PyTile>>();

    _tiles.sort_by_key(|t| t.xyz.z);

    // Check to see if a tile and its parent both already exist.
    // Ensure that tiles are sorted by zoom so parents are encountered first.
    // If so, discard the child (it's covered in the parent)
    let mut root_set: HashSet<PyTile> = HashSet::new();
    for tile in &_tiles {
        let mut is_new_tile = true;
        for i in 0..tile.xyz.z {
            let supertile = tile.parent(Some(i));
            if root_set.contains(&supertile) {
                is_new_tile = false;
                break;
            }
        }
        if is_new_tile {
            root_set.insert(*tile);
        }
    }

    // Repeatedly run merge until no further simplification is possible.
    let mut is_merging = true;
    while is_merging {
        let (new_set, changed) = merge(&root_set);
        root_set = new_set;
        is_merging = changed;
    }
    Ok(root_set)
}

#[pyfunction]
fn coords(py: Python, obj: &PyAny) -> PyResult<Vec<(f64, f64)>> {
    let coordsvec = _coords(py, obj);
    match coordsvec {
        Ok(coordsvec) => {
            let coordsvec = coordsvec.iter.map(|(lng, lat)| (lng, lat)).collect();
            Ok(coordsvec)
        }
        Err(e) => Err(e),
    }
}

impl Iterator for utiles::LngLat {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let lng = self.xy.x;
        let lat = self.xy.y;
        self.xy.x += 1.0;
        self.xy.y += 1.0;
        Some((lng, lat))
    }
}

#[pyfunction]
fn geojson_bounds(py: Python, obj: &PyAny) -> PyResult<PyLngLatBbox> {
    let coordsvec = _coords(py, obj);
    match coordsvec {
        Ok(coordsvec) => {
            let coordsvec: Vec<(f64, f64)> =
                coordsvec.iter.map(|(lng, lat)| (lng, lat)).collect();
            let mut bbox: (f64, f64, f64, f64) = (180.0, 90.0, -180.0, -90.0);

            for (lng, lat) in coordsvec {
                if lat <= -90.0 || lat >= 90.0 {
                    return Err(PyErr::new::<PyValueError, _>(format!(
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
        Err(e) => Err(e),
    }
}

#[pyfunction]
fn feature(
    py: Python,
    tile: PyTileLike,
    // (u32, u32, u8),
    fid: Option<String>,
    props: Option<HashMap<String, &PyAny>>,
    projected: Option<String>,
    buffer: Option<f64>,
    precision: Option<i32>,
) -> PyResult<HashMap<String, PyObject>> {
    // Convert the arguments to Rust values
    let pytile: PyTile = tile.into();
    let f = pytile.feature(py, fid, props, projected, buffer, precision)?;
    Ok(f)
}

/// Utiles python module
#[pymodule]
fn libutiles(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add("__version_lib__", env!("CARGO_PKG_VERSION"))?;
    m.add("__build_profile__", env!("PROFILE"))?;

    // mercantile functions
    m.add_function(wrap_pyfunction!(parse_tile_arg, m)?)?;
    m.add_function(wrap_pyfunction!(_parse_tile_arg, m)?)?;
    m.add_function(wrap_pyfunction!(minmax, m)?)?;
    m.add_function(wrap_pyfunction!(ul, m)?)?;
    m.add_function(wrap_pyfunction!(bounds, m)?)?;
    m.add_function(wrap_pyfunction!(xy, m)?)?;
    m.add_function(wrap_pyfunction!(_xy, m)?)?;
    m.add_function(wrap_pyfunction!(lnglat, m)?)?;
    m.add_function(wrap_pyfunction!(xy_bounds, m)?)?;
    m.add_function(wrap_pyfunction!(tile, m)?)?;
    m.add_function(wrap_pyfunction!(parent, m)?)?;
    m.add_function(wrap_pyfunction!(quadkey, m)?)?;
    m.add_function(wrap_pyfunction!(quadkey_to_tile, m)?)?;
    m.add_function(wrap_pyfunction!(children, m)?)?;
    m.add_function(wrap_pyfunction!(neighbors, m)?)?;
    m.add_function(wrap_pyfunction!(tiles, m)?)?;
    m.add_function(wrap_pyfunction!(bounding_tile, m)?)?;
    m.add_function(wrap_pyfunction!(truncate_lnglat, m)?)?;
    m.add_function(wrap_pyfunction!(_coords, m)?)?;
    m.add_function(wrap_pyfunction!(coords, m)?)?;
    // m.add_function(wrap_pyfunction!(merge, m)?)?;
    m.add_function(wrap_pyfunction!(simplify, m)?)?;
    m.add_function(wrap_pyfunction!(geojson_bounds, m)?)?;
    m.add_function(wrap_pyfunction!(feature, m)?)?;

    // utiles functions
    m.add_function(wrap_pyfunction!(tiles_list, m)?)?;
    m.add_function(wrap_pyfunction!(xyz, m)?)?;
    m.add_function(wrap_pyfunction!(parse_tiles, m)?)?;
    m.add_function(wrap_pyfunction!(xyz2quadkey, m)?)?;
    m.add_function(wrap_pyfunction!(quadkey2xyz, m)?)?;
    m.add_function(wrap_pyfunction!(from_tuple, m)?)?;
    m.add_function(wrap_pyfunction!(pmtileid, m)?)?;
    m.add_function(wrap_pyfunction!(from_pmtileid, m)?)?;

    // tiletype
    m.add_function(wrap_pyfunction!(tiletype, m)?)?;
    m.add_function(wrap_pyfunction!(tiletype_str, m)?)?;
    m.add_function(wrap_pyfunction!(tiletype2headers, m)?)?;
    m.add("TILETYPE_UNKNOWN", libtiletype::TILETYPE_UNKNOWN)?; // 0
    m.add("TILETYPE_GIF", libtiletype::TILETYPE_GIF)?; // 1
    m.add("TILETYPE_JPG", libtiletype::TILETYPE_JPG)?; // 2
    m.add("TILETYPE_PBF", libtiletype::TILETYPE_PBF)?; // 3
    m.add("TILETYPE_PNG", libtiletype::TILETYPE_PNG)?; // 4
    m.add("TILETYPE_WEBP", libtiletype::TILETYPE_WEBP)?; // 5

    // m.add_class::<TileTuple>()?;
    m.add_class::<PyTile>()?;
    m.add_class::<PyLngLat>()?;
    m.add_class::<PyLngLatBbox>()?;
    m.add_class::<PyBbox>()?;
    Ok(())
}
