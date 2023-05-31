use pyo3::prelude::*;
use std::collections::HashMap;

use crate::pyutiles::pyiters::IntIterator;
use crate::pyutiles::tuple_slice;
use crate::utiles::{BBox, Tile};

use crate::{utiles, TileTuple};
use pyo3::basic::CompareOp;
use pyo3::types::PyType;

use pyo3::exceptions::PyValueError;
use pyo3::{
    exceptions, pyclass, pymethods, IntoPy, Py, PyAny, PyErr, PyObject, PyRef,
    PyResult, Python,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;

use crate::pyutiles::pylnglat::PyLngLat;
use crate::pyutiles::pylnglatbbox::PyLngLatBbox;
use std::hash::{Hash, Hasher};

/// `PyTile` macro to create a new tile.
///  - do you need this? probably not
///  - Did I write to to figure out how to write a macro? yes
#[macro_export]
macro_rules! pytile {
    ($x:expr, $y:expr, $z:expr) => {
        PyTile {
            xyz: utile!($x, $y, $z),
        }
    };
}

#[pyclass(name = "Tile", sequence)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash, Copy)]
pub struct PyTile {
    pub xyz: Tile,
}

#[pymethods]
impl PyTile {
    #[new]
    pub fn new(x: u32, y: u32, z: u8) -> Self {
        Self {
            xyz: Tile::new(x, y, z),
        }
    }

    pub fn valid(&self) -> bool {
        self.xyz.valid()
    }

    pub fn json_obj(&self) -> String {
        let json = serde_json::to_string(&self.xyz);
        match json {
            Ok(json) => json,
            Err(e) => format!("Error: {e}"),
        }
    }

    pub fn json_arr(&self) -> String {
        let s = format!("[{}, {}, {}]", self.xyz.x, self.xyz.y, self.xyz.z);
        s
    }

    pub fn json(&self, obj: Option<bool>) -> String {
        if obj.unwrap_or(true) {
            self.json_obj()
        } else {
            self.json_arr()
        }
    }

    pub fn asdict(&self) -> HashMap<&str, u32> {
        let mut map = HashMap::new();
        map.insert("x", self.xyz.x());
        map.insert("y", self.xyz.y());
        map.insert("z", u32::from(self.xyz.z()));
        map
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<IntIterator>> {
        let iter = IntIterator {
            iter: Box::new(
                vec![slf.xyz.x, slf.xyz.y, u32::from(slf.xyz.z)].into_iter(),
            ),
        };
        Py::new(slf.py(), iter)
    }

    pub fn fmt_zxy(&self) -> String {
        self.xyz.fmt_zxy()
    }

    pub fn fmt_zxy_ext(&self, ext: &str) -> String {
        self.xyz.fmt_zxy_ext(ext)
    }

    #[classmethod]
    pub fn from_quadkey(_cls: &PyType, quadkey: String) -> PyResult<Self> {
        let xyz = Tile::from_quadkey(&quadkey);
        match xyz {
            Ok(xyz) => Ok(PyTile::from(xyz)),
            Err(e) => Err(PyErr::new::<PyValueError, _>(format!("Error: {e}"))),
        }
    }

    #[classmethod]
    pub fn from_qk(_cls: &PyType, quadkey: String) -> PyResult<Self> {
        let xyz = Tile::from_quadkey(&quadkey);
        match xyz {
            Ok(xyz) => Ok(PyTile::from(xyz)),
            Err(e) => Err(PyErr::new::<PyValueError, _>(format!("Error: {e}"))),
        }
    }

    pub fn quadkey(&self) -> String {
        self.xyz.quadkey()
    }

    pub fn qk(&self) -> String {
        self.xyz.quadkey()
    }

    #[classmethod]
    pub fn from_pmtileid(_cls: &PyType, tileid: u64) -> PyResult<Self> {
        let xyz = Tile::from_pmtileid(tileid);
        Ok(PyTile::from(xyz))
    }

    pub fn pmtileid(&self) -> u64 {
        self.xyz.pmtileid()
    }

    pub fn parent_pmtileid(&self) -> u64 {
        self.xyz.parent_id()
    }

    #[classmethod]
    pub fn from_lnglat_zoom(
        _cls: &PyType,
        lng: f64,
        lat: f64,
        zoom: u8,
        truncate: Option<bool>,
    ) -> PyResult<Self> {
        let xyz = Tile::from_lnglat_zoom(lng, lat, zoom, truncate);
        Ok(PyTile::from(xyz))
    }

    pub fn __repr__(&self) -> String {
        format!("Tile(x={}, y={}, z={})", self.xyz.x, self.xyz.y, self.xyz.z)
    }

    #[getter]
    pub fn x(&self) -> PyResult<u32> {
        Ok(self.xyz.x)
    }

    #[getter]
    pub fn y(&self) -> PyResult<u32> {
        Ok(self.xyz.y)
    }

    #[getter]
    pub fn z(&self) -> PyResult<u8> {
        Ok(self.xyz.z)
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn __invert__(&self) -> Self {
        let y = utiles::flipy(self.xyz.y, self.xyz.z);
        Self {
            xyz: Tile::new(self.xyz.x, y, self.xyz.z),
        }
    }

    pub fn flipy(&self) -> Self {
        self.__invert__()
    }

    pub fn __len__(&self) -> usize {
        3
    }

    pub fn members(&self) -> Vec<u32> {
        vec![self.xyz.x, self.xyz.y, u32::from(self.xyz.z)]
    }

    pub fn __getitem__(
        &self,
        idx: tuple_slice::SliceOrInt,
        _py: Python<'_>,
    ) -> PyResult<tuple_slice::TupleSliceResult<u32>> {
        match idx {
            tuple_slice::SliceOrInt::Slice(slice) => {
                let psi = slice.indices(3)?;
                let (start, stop, step) = (psi.start, psi.stop, psi.step);
                let m: Vec<u32> = self.members()[start as usize..stop as usize]
                    .iter()
                    .step_by(step as usize)
                    .copied()
                    .collect();
                let m = tuple_slice::TupleSliceResult::Slice(m);
                Ok(m)
            }
            tuple_slice::SliceOrInt::Int(idx) => match idx {
                0 => Ok(tuple_slice::TupleSliceResult::It(self.xyz.x)),
                1 => Ok(tuple_slice::TupleSliceResult::It(self.xyz.y)),
                2 => Ok(tuple_slice::TupleSliceResult::It(u32::from(self.xyz.z))),
                -1 => Ok(tuple_slice::TupleSliceResult::It(u32::from(self.xyz.z))),
                -2 => Ok(tuple_slice::TupleSliceResult::It(self.xyz.y)),
                -3 => Ok(tuple_slice::TupleSliceResult::It(self.xyz.x)),
                3 => Err(PyErr::new::<exceptions::PyStopIteration, _>("")),
                _ => panic!("Index {idx} out of range for tile"),
            },
        }
    }

    pub fn bounds(&self) -> PyLngLatBbox {
        let (west, south, east, north) = self.xyz.bounds();
        PyLngLatBbox {
            bbox: BBox {
                north,
                south,
                east,
                west,
            },
        }
    }

    pub fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.xyz.x.hash(&mut hasher);
        self.xyz.y.hash(&mut hasher);
        self.xyz.z.hash(&mut hasher);
        hasher.finish()
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        self.xyz.x == other.xyz.x
            && self.xyz.y == other.xyz.y
            && self.xyz.z == other.xyz.z
    }

    pub fn __richcmp__(
        &self,
        other: &PyAny,
        op: CompareOp,
        py: Python<'_>,
    ) -> PyObject {
        // fn __richcmp__(&self, other: PyAny, op: CompareOp, py: Python<'_>) -> PyObject {
        let maybetuple = other.extract::<(u32, u32, u8)>();
        if let Ok(tuple) = maybetuple {
            match op {
                CompareOp::Lt => ((self.xyz.x < tuple.0)
                    && (self.xyz.y < tuple.1)
                    && (self.xyz.z < tuple.2))
                    .into_py(py),
                CompareOp::Eq => ((self.xyz.x == tuple.0)
                    && (self.xyz.y == tuple.1)
                    && (self.xyz.z == tuple.2))
                    .into_py(py),
                CompareOp::Ne => ((self.xyz.x != tuple.0)
                    || (self.xyz.y != tuple.1)
                    || (self.xyz.z != tuple.2))
                    .into_py(py),
                _ => py.NotImplemented(),
            }
        } else {
            let other = other.extract::<PyRef<PyTile>>().unwrap();
            match op {
                CompareOp::Lt => ((self.xyz.x < other.xyz.x)
                    && (self.xyz.y < other.xyz.y)
                    && (self.xyz.z < other.xyz.z))
                    .into_py(py),
                CompareOp::Eq => ((self.xyz.x == other.xyz.x)
                    && (self.xyz.y == other.xyz.y)
                    && (self.xyz.z == other.xyz.z))
                    .into_py(py),
                CompareOp::Ne => ((self.xyz.x != other.xyz.x)
                    || (self.xyz.y != other.xyz.y)
                    || (self.xyz.z != other.xyz.z))
                    .into_py(py),
                _ => py.NotImplemented(),
            }
        }
    }
    pub fn ul(&self) -> PyLngLat {
        self.xyz.ul().into()
    }
    pub fn ll(&self) -> PyLngLat {
        self.xyz.ll().into()
    }

    pub fn ur(&self) -> PyLngLat {
        self.xyz.ur().into()
    }

    pub fn lr(&self) -> PyLngLat {
        self.xyz.lr().into()
    }

    pub fn center(&self) -> PyLngLat {
        self.xyz.center().into()
    }

    pub fn parent(&self, n: Option<u8>) -> Self {
        self.xyz.parent(n).into()
    }

    pub fn children(&self, zoom: Option<u8>) -> Vec<Self> {
        let xyzs = self.xyz.children(zoom);
        xyzs.into_iter().map(Self::from).collect()
    }

    pub fn siblings(&self) -> Vec<Self> {
        self.xyz.siblings().into_iter().map(Self::from).collect()
    }

    pub fn neighbors(&self) -> Vec<Self> {
        self.xyz.neighbors().into_iter().map(Self::from).collect()
    }

    pub fn tuple(&self) -> (u32, u32, u8) {
        self.xyz.into()
    }

    pub fn feature(
        &self,
        py: Python,
        // tile:  PyTileLike,
        // (u32, u32, u8),
        fid: Option<String>,
        props: Option<HashMap<String, &PyAny>>,
        projected: Option<String>,
        buffer: Option<f64>,
        precision: Option<i32>,
    ) -> PyResult<HashMap<String, PyObject>> {
        // Convert the arguments to Rust values
        // let pytile: PyTile = tile.into();
        // let tile = pytile.tuple();
        let (x, y, z) = self.tuple();
        let fid = fid.unwrap_or_default();
        let props = props.unwrap_or_default();
        let projected = projected.unwrap_or_else(|| "geographic".to_string());
        let buffer = buffer.unwrap_or(0.0);
        let precision = precision.unwrap_or(-1);

        // Compute the bounds
        let (west, south, east, north) = utiles::bounds(x, y, z);

        // Handle projected coordinates
        let (mut west, mut south, mut east, mut north) = match projected.as_str() {
            "mercator" => {
                // let (east_merc, north_merc) = utiles::xy(east, north, Some(false));
                let (west_merc, south_merc) = utiles::xy(west, south, None);
                let (east_merc, north_merc) = utiles::xy(east, north, None);
                (west_merc, south_merc, east_merc, north_merc)
            }
            _ => (west, south, east, north),
        };

        // Apply buffer
        west -= buffer;
        south -= buffer;
        east += buffer;
        north += buffer;

        // Apply precision
        if precision >= 0 {
            let precision_factor = 10_f64.powi(precision);
            west = (west * precision_factor).round() / precision_factor;
            south = (south * precision_factor).round() / precision_factor;
            east = (east * precision_factor).round() / precision_factor;
            north = (north * precision_factor).round() / precision_factor;
        }

        // Compute bbox and geometry
        let bbox = [
            west.min(east),
            south.min(north),
            west.max(east),
            south.max(north),
        ];
        let geometry_coordinates = vec![vec![
            [west, south],
            [west, north],
            [east, north],
            [east, south],
            [west, south],
        ]];

        let geometry_items = vec![
            ("type".to_string(), "Polygon".to_object(py)),
            (
                "coordinates".to_string(),
                geometry_coordinates.to_object(py),
            ),
        ]
        .into_iter()
        .collect::<HashMap<String, PyObject>>();

        // Create the feature dictionary
        let xyz = format!("({x}, {y}, {z})").into_py(py);
        let mut feature_dict = HashMap::new();
        feature_dict.insert("type".to_string(), "Feature".to_object(py));
        feature_dict.insert("bbox".to_string(), bbox.to_object(py));
        feature_dict.insert("id".to_string(), xyz.to_object(py));
        feature_dict.insert("geometry".to_string(), geometry_items.to_object(py));

        // Create the properties dictionary
        let mut properties_dict: HashMap<String, Py<PyAny>> = HashMap::new();
        properties_dict
            .insert("title".to_string(), format!("XYZ tile {xyz}").into_py(py));
        if !props.is_empty() {
            let props: PyResult<Vec<(String, Py<PyAny>)>> = props
                .into_iter()
                .map(|(k, v)| Ok((k, v.into_py(py))))
                .collect();
            properties_dict.extend(props?);
        }
        feature_dict.insert("properties".to_string(), properties_dict.to_object(py));

        // Add the feature id if provided
        if !fid.is_empty() {
            feature_dict.insert("id".to_string(), fid.to_object(py));
        }
        Ok(feature_dict)
    }
}

impl From<Tile> for PyTile {
    fn from(xyz: Tile) -> Self {
        Self { xyz }
    }
}

impl From<(u32, u32, u32)> for PyTile {
    fn from(xyz: (u32, u32, u32)) -> Self {
        Self {
            xyz: Tile::new(xyz.0, xyz.1, xyz.2 as u8),
        }
    }
}

impl From<TileTuple> for PyTile {
    fn from(tile: TileTuple) -> Self {
        Self {
            xyz: Tile::new(tile.0, tile.1, tile.2),
        }
    }
}

impl From<PyTile> for Tile {
    fn from(val: PyTile) -> Self {
        val.xyz
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utile;

    #[test]
    fn test_pytile_macro() {
        let tile = pytile!(0, 0, 0);
        assert_eq!(tile.xyz.x, 0);
        assert_eq!(tile.xyz.y, 0);
        assert_eq!(tile.xyz.z, 0);
    }
}
