use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};

use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use pyo3::{
    exceptions, pyclass, pymethods, IntoPy, Py, PyAny, PyErr, PyObject, PyRef,
    PyResult, Python,
};
use serde::Serialize;
use utiles::bbox::BBox;
use utiles::projection::Projection;
use utiles::tile::{FeatureOptions, Tile};
use utiles::{TileLike, TileParent};

use crate::pyutiles::pyiters::IntIterator;
use crate::pyutiles::pylnglat::PyLngLat;
use crate::pyutiles::pylnglatbbox::PyLngLatBbox;
use crate::pyutiles::pytile_tuple::TileTuple;
use crate::pyutiles::tuple_slice;

/// `PyTile` macro to create a new tile.
///  - do you need this? probably not
///  - Did I write to to figure out how to write a macro? yes
#[macro_export]
macro_rules! pytile {
    ($x:expr, $y:expr, $z:expr) => {
        PyTile {
            xyz: utiles::utile!($x, $y, $z),
        }
    };
}

#[pyclass(name = "Tile", module = "utiles._utiles", sequence)]
#[derive(Clone, Debug, PartialEq, Serialize, Eq, Hash, Copy)]
pub struct PyTile {
    pub xyz: Tile,
}

// #[derive(FromPyObject)]
// pub enum PyTileOrTuple {
//     Tile(PyTile),
//     Tuple(TileTuple),
// }

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
        json.unwrap_or_else(|e| format!("Error: {e}"))
    }

    pub fn json_arr(&self) -> String {
        let s = format!("[{}, {}, {}]", self.xyz.x, self.xyz.y, self.xyz.z);
        s
    }

    #[pyo3(signature = (obj = true))]
    pub fn json(&self, obj: Option<bool>) -> String {
        if obj.unwrap_or(true) {
            self.json_obj()
        } else {
            self.json_arr()
        }
    }

    pub fn asdict(&self) -> BTreeMap<&str, u32> {
        let mut map = BTreeMap::new();
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

    #[pyo3(signature = (sep = None))]
    pub fn fmt_zxy(&self, sep: Option<&str>) -> String {
        self.xyz.fmt_zxy(sep)
    }

    #[pyo3(signature = (ext = "", sep = None))]
    pub fn fmt_zxy_ext(&self, ext: &str, sep: Option<&str>) -> String {
        self.xyz.fmt_zxy_ext(ext, sep)
    }

    #[classmethod]
    pub fn from_quadkey(_cls: &Bound<'_, PyType>, quadkey: String) -> PyResult<Self> {
        let xyz = Tile::from_quadkey(&quadkey);
        match xyz {
            Ok(xyz) => Ok(PyTile::from(xyz)),
            Err(e) => Err(PyErr::new::<PyValueError, _>(format!("Error: {e}"))),
        }
    }

    #[classmethod]
    pub fn from_qk(_cls: &Bound<'_, PyType>, quadkey: String) -> PyResult<Self> {
        let xyz = Tile::from_quadkey(&quadkey);
        match xyz {
            Ok(xyz) => Ok(PyTile::from(xyz)),
            Err(e) => Err(PyErr::new::<PyValueError, _>(format!("Error: {e}"))),
        }
    }

    #[classmethod]
    pub fn from_row_major_id(_cls: &Bound<'_, PyType>, row_major_id: u64) -> Self {
        let xyz = Tile::from_row_major_id(row_major_id);
        PyTile::from(xyz)
    }

    #[classmethod]
    pub fn from_rmid(_cls: &Bound<'_, PyType>, row_major_id: u64) -> Self {
        let xyz = Tile::from_row_major_id(row_major_id);
        PyTile::from(xyz)
    }

    pub fn quadkey(&self) -> String {
        self.xyz.quadkey()
    }

    pub fn qk(&self) -> String {
        self.xyz.quadkey()
    }

    #[classmethod]
    pub fn from_pmtileid(_cls: &Bound<'_, PyType>, tileid: u64) -> Self {
        let xyz = Tile::from_pmtileid(tileid);
        PyTile::from(xyz)
    }

    pub fn pmtileid(&self) -> u64 {
        self.xyz.pmtileid()
    }

    pub fn parent_pmtileid(&self) -> u64 {
        self.xyz.parent_pmtileid()
    }

    pub fn row_major_id(&self) -> u64 {
        self.xyz.row_major_id()
    }

    pub fn rmid(&self) -> u64 {
        self.xyz.row_major_id()
    }

    #[classmethod]
    #[pyo3(signature = (lng, lat, zoom, truncate = None))]
    pub fn from_lnglat_zoom(
        _cls: &Bound<'_, PyType>,
        lng: f64,
        lat: f64,
        zoom: u8,
        truncate: Option<bool>,
    ) -> PyResult<Self> {
        let xyz = Tile::from_lnglat_zoom(lng, lat, zoom, truncate)
            .map_err(|e| PyErr::new::<PyValueError, _>(format!("Error: {e}")))?;
        Ok(PyTile::from(xyz))
    }

    pub fn __repr__(&self) -> String {
        format!("Tile(x={}, y={}, z={})", self.xyz.x, self.xyz.y, self.xyz.z)
    }

    #[getter]
    pub fn x(&self) -> u32 {
        self.xyz.x
    }

    #[getter]
    pub fn y(&self) -> u32 {
        self.xyz.y
    }

    #[getter]
    pub fn z(&self) -> u8 {
        self.xyz.z
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
                0 | -3 => Ok(tuple_slice::TupleSliceResult::It(self.xyz.x)),
                1 | -2 => Ok(tuple_slice::TupleSliceResult::It(self.xyz.y)),
                2 | -1 => Ok(tuple_slice::TupleSliceResult::It(u32::from(self.xyz.z))),
                3 => Err(PyErr::new::<exceptions::PyStopIteration, _>("")),
                _ => panic!("Index {idx} out of range for tile"),
            },
        }
    }

    pub fn bounds(&self) -> PyLngLatBbox {
        let (west, south, east, north) = self.xyz.bounds();
        PyLngLatBbox {
            bbox: BBox {
                west,
                south,
                east,
                north,
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

    pub fn __richcmp__(
        &self,
        other: &Bound<'_, PyAny>,
        op: CompareOp,
        py: Python<'_>,
    ) -> PyObject {
        let is_pytile = other.is_instance_of::<PyTile>();
        if is_pytile {
            let maybe_pytile = other.extract::<PyTile>();
            match maybe_pytile {
                Ok(other) => match op {
                    CompareOp::Eq => ((self.xyz.x == other.xyz.x)
                        && (self.xyz.y == other.xyz.y)
                        && (self.xyz.z == other.xyz.z))
                        .into_py(py),
                    CompareOp::Ne => ((self.xyz.x != other.xyz.x)
                        || (self.xyz.y != other.xyz.y)
                        || (self.xyz.z != other.xyz.z))
                        .into_py(py),
                    CompareOp::Lt => ((self.xyz.x < other.xyz.x)
                        && (self.xyz.y < other.xyz.y)
                        && (self.xyz.z < other.xyz.z))
                        .into_py(py),
                    CompareOp::Gt => ((self.xyz.x > other.xyz.x)
                        && (self.xyz.y > other.xyz.y)
                        && (self.xyz.z > other.xyz.z))
                        .into_py(py),
                    CompareOp::Ge => ((self.xyz.x >= other.xyz.x)
                        && (self.xyz.y >= other.xyz.y)
                        && (self.xyz.z >= other.xyz.z))
                        .into_py(py),
                    CompareOp::Le => ((self.xyz.x <= other.xyz.x)
                        && (self.xyz.y <= other.xyz.y)
                        && (self.xyz.z <= other.xyz.z))
                        .into_py(py),
                },
                Err(_) => py.NotImplemented(),
            }
        } else if let Ok(tuple) = other.extract::<(u32, u32, u8)>() {
            match op {
                CompareOp::Eq => ((self.xyz.x == tuple.0)
                    && (self.xyz.y == tuple.1)
                    && (self.xyz.z == tuple.2))
                    .into_py(py),
                CompareOp::Ne => ((self.xyz.x != tuple.0)
                    || (self.xyz.y != tuple.1)
                    || (self.xyz.z != tuple.2))
                    .into_py(py),
                CompareOp::Lt => ((self.xyz.x < tuple.0)
                    && (self.xyz.y < tuple.1)
                    && (self.xyz.z < tuple.2))
                    .into_py(py),
                CompareOp::Gt => ((self.xyz.x > tuple.0)
                    && (self.xyz.y > tuple.1)
                    && (self.xyz.z > tuple.2))
                    .into_py(py),
                CompareOp::Ge => ((self.xyz.x >= tuple.0)
                    && (self.xyz.y >= tuple.1)
                    && (self.xyz.z >= tuple.2))
                    .into_py(py),
                CompareOp::Le => ((self.xyz.x <= tuple.0)
                    && (self.xyz.y <= tuple.1)
                    && (self.xyz.z <= tuple.2))
                    .into_py(py),
            }
        } else {
            py.NotImplemented()
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

    #[pyo3(signature = (n = None))]
    pub fn parent(&self, n: Option<u8>) -> Self {
        self.xyz.parent(n).into()
    }

    #[pyo3(signature = (zoom = None))]
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

    #[pyo3(
        signature = (fid = None, props = None, projected = None, buffer = None, precision = None)
    )]
    pub fn feature(
        &self,
        py: Python,
        fid: Option<String>,
        props: Option<HashMap<String, Bound<PyAny>>>,
        projected: Option<String>,
        buffer: Option<f64>,
        precision: Option<i32>,
    ) -> PyResult<HashMap<String, PyObject>> {
        let projection = if let Some(projected) = projected {
            Projection::try_from(projected)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("Error: {e}")))?
        } else {
            Projection::Geographic
        };
        let feature_opts = FeatureOptions {
            buffer,
            fid,
            precision,
            projection,
            props: None,
        };
        let tfeat = self
            .xyz
            .feature(&feature_opts)
            .map_err(|e| PyErr::new::<PyValueError, _>(format!("Error: {e}")))?;

        // feature that will become python object
        let mut feature_dict = HashMap::new();
        let bbox_vec = vec![tfeat.bbox.0, tfeat.bbox.1, tfeat.bbox.2, tfeat.bbox.3];
        let geometry_items = vec![
            ("type".to_string(), tfeat.geometry.type_.to_object(py)),
            (
                "coordinates".to_string(),
                tfeat.geometry.coordinates.to_object(py),
            ),
        ]
        .into_iter()
        .collect::<HashMap<String, PyObject>>();
        // Create the properties dictionary
        let mut properties_dict: HashMap<String, Py<PyAny>> = HashMap::new();
        let (x, y, z) = self.tuple();
        let xyz_tuple_string = format!("({x}, {y}, {z})").into_py(py);

        properties_dict.insert(
            "title".to_string(),
            format!("XYZ tile {xyz_tuple_string}").into_py(py),
        );
        if let Some(props) = props {
            let props: PyResult<Vec<(String, Py<PyAny>)>> = props
                .into_iter()
                .map(|(k, v)| Ok((k, v.into_py(py))))
                .collect();
            properties_dict.extend(props?);
        }
        feature_dict.insert("type".to_string(), "Feature".to_object(py));
        feature_dict.insert("bbox".to_string(), bbox_vec.to_object(py));
        feature_dict.insert("id".to_string(), tfeat.id.to_object(py));
        feature_dict.insert("geometry".to_string(), geometry_items.to_object(py));
        feature_dict.insert("properties".to_string(), properties_dict.to_object(py));
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

impl TileLike for PyTile {
    fn x(&self) -> u32 {
        self.xyz.x
    }

    fn y(&self) -> u32 {
        self.xyz.y
    }

    fn z(&self) -> u8 {
        self.xyz.z
    }
}

impl TileLike for &PyTile {
    fn x(&self) -> u32 {
        self.xyz.x
    }

    fn y(&self) -> u32 {
        self.xyz.y
    }

    fn z(&self) -> u8 {
        self.xyz.z
    }
}

impl TileParent for PyTile {
    fn parent(&self, zoom: Option<u8>) -> Self {
        self.parent(zoom)
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

    #[test]
    fn test_pytile_macro() {
        let tile = pytile!(0, 0, 0);
        assert_eq!(tile.xyz.x, 0);
        assert_eq!(tile.xyz.y, 0);
        assert_eq!(tile.xyz.z, 0);
    }
}
