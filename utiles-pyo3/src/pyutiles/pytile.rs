use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use pyo3::basic::CompareOp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyNotImplemented, PyTuple, PyType};
use pyo3::{
    exceptions, intern, pyclass, pymethods, IntoPyObjectExt, Py, PyAny, PyErr, PyRef,
    PyResult, Python,
};
use serde::Serialize;
use utiles::bbox::BBox;
use utiles::projection::Projection;
use utiles::tile::{FeatureOptions, Tile};
use utiles::{TileChildren1, TileLike, TileParent};

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

#[pyclass(name = "Tile", module = "utiles._utiles", sequence, frozen)]
#[derive(Clone, Debug, PartialEq, Serialize, Eq, Hash, Copy)]
pub struct PyTile {
    pub xyz: Tile,
}

#[pymethods]
impl PyTile {
    #[new]
    pub fn py_new(x: u32, y: u32, z: u8) -> Self {
        Self {
            xyz: Tile::new(x, y, z),
        }
    }

    fn __getnewargs__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, [self.xyz.x(), self.xyz.y(), self.xyz.z() as u32])
    }

    pub fn valid(&self) -> bool {
        self.xyz.valid()
    }

    pub fn json_obj(&self) -> String {
        let json = serde_json::to_string(&self.xyz);
        json.unwrap_or_else(|e| format!("Error: {e}"))
    }

    pub fn json_arr(&self) -> String {
        format!("[{}, {}, {}]", self.xyz.x, self.xyz.y, self.xyz.z)
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

    pub fn parent_pmtileid(&self) -> Option<u64> {
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

    pub fn __getitem__<'py>(
        &self,
        idx: tuple_slice::SliceOrInt,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        match idx {
            tuple_slice::SliceOrInt::Slice(slice) => {
                let psi = slice.indices(3)?;
                let (start, stop, step) = (psi.start, psi.stop, psi.step);
                let m: Vec<u32> = self.members()[start as usize..stop as usize]
                    .iter()
                    .step_by(step as usize)
                    .copied()
                    .collect();
                let tuple = PyTuple::new(py, m).map(Bound::into_any).map_err(|e| {
                    PyErr::new::<PyValueError, _>(format!("Error: {e}"))
                })?;
                Ok(tuple)
            }
            tuple_slice::SliceOrInt::Int(idx) => match idx {
                0 | -3 => self.xyz.x().into_bound_py_any(py),
                1 | -2 => self.xyz.y().into_bound_py_any(py),
                2 | -1 => {
                    let r = u32::from(self.xyz.z)
                        .into_pyobject(py)
                        .map(Bound::into_any)?;
                    Ok(r)
                }
                3 => Err(PyErr::new::<exceptions::PyStopIteration, _>("")),
                _ => Err(PyErr::new::<exceptions::PyIndexError, _>(
                    "Index out of range",
                )),
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
    ) -> PyResult<bool> {
        let is_pytile = other.is_instance_of::<PyTile>();
        if is_pytile {
            let maybe_pytile = other.extract::<PyTile>();
            match maybe_pytile {
                Ok(other) => {
                    let b = match op {
                        CompareOp::Eq => {
                            (self.xyz.x == other.xyz.x)
                                && (self.xyz.y == other.xyz.y)
                                && (self.xyz.z == other.xyz.z)
                        }
                        CompareOp::Ne => {
                            (self.xyz.x != other.xyz.x)
                                || (self.xyz.y != other.xyz.y)
                                || (self.xyz.z != other.xyz.z)
                        }
                        CompareOp::Lt => {
                            (self.xyz.x < other.xyz.x)
                                && (self.xyz.y < other.xyz.y)
                                && (self.xyz.z < other.xyz.z)
                        }
                        CompareOp::Gt => {
                            (self.xyz.x > other.xyz.x)
                                && (self.xyz.y > other.xyz.y)
                                && (self.xyz.z > other.xyz.z)
                        }
                        CompareOp::Ge => {
                            (self.xyz.x >= other.xyz.x)
                                && (self.xyz.y >= other.xyz.y)
                                && (self.xyz.z >= other.xyz.z)
                        }
                        CompareOp::Le => {
                            (self.xyz.x <= other.xyz.x)
                                && (self.xyz.y <= other.xyz.y)
                                && (self.xyz.z <= other.xyz.z)
                        }
                    };
                    Ok(b)
                }
                Err(_) => Err(PyErr::new::<PyNotImplemented, _>("Should not happen")),
            }
        } else if let Ok(tuple) = other.extract::<(u32, u32, u8)>() {
            let r = match op {
                CompareOp::Eq => {
                    (self.xyz.x == tuple.0)
                        && (self.xyz.y == tuple.1)
                        && (self.xyz.z == tuple.2)
                }
                CompareOp::Ne => {
                    (self.xyz.x != tuple.0)
                        || (self.xyz.y != tuple.1)
                        || (self.xyz.z != tuple.2)
                }
                CompareOp::Lt => {
                    (self.xyz.x < tuple.0)
                        && (self.xyz.y < tuple.1)
                        && (self.xyz.z < tuple.2)
                }
                CompareOp::Gt => {
                    (self.xyz.x > tuple.0)
                        && (self.xyz.y > tuple.1)
                        && (self.xyz.z > tuple.2)
                }
                CompareOp::Ge => {
                    (self.xyz.x >= tuple.0)
                        && (self.xyz.y >= tuple.1)
                        && (self.xyz.z >= tuple.2)
                }
                CompareOp::Le => {
                    (self.xyz.x <= tuple.0)
                        && (self.xyz.y <= tuple.1)
                        && (self.xyz.z <= tuple.2)
                }
            };
            Ok(r)
        } else {
            match op {
                CompareOp::Eq => Ok(false),
                CompareOp::Ne => Ok(true),
                _ => Err(PyErr::new::<PyNotImplemented, _>(
                    "Comparison not implemented for PyTile",
                )),
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

    #[pyo3(signature = (n = None))]
    pub fn parent(&self, n: Option<u8>) -> Option<Self> {
        self.xyz.parent(n).map(PyTile::from)
    }

    #[pyo3(signature = (zoom = None, *, zorder = None))]
    pub fn children(&self, zoom: Option<u8>, zorder: Option<bool>) -> Vec<Self> {
        let zorder = zorder.unwrap_or(false);
        let xyzs = {
            if zorder {
                self.xyz.children_zorder(zoom)
            } else {
                self.xyz.children(zoom)
            }
        };
        xyzs.into_iter().map(Self::from).collect()
    }

    pub fn children1(&self) -> [Self; 4] {
        let direct_child_tiles = self.xyz.children1();
        direct_child_tiles.map(Self::from)
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
    pub fn feature<'py>(
        &self,
        py: Python<'py>,
        fid: Option<String>,
        props: Option<Bound<'py, PyDict>>,
        projected: Option<String>,
        buffer: Option<f64>,
        precision: Option<i32>,
    ) -> PyResult<Bound<'py, PyDict>> {
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
        let feature_dict = PyDict::new(py);
        // let mut feature_dict = HashMap::new();
        let bbox_vec = vec![tfeat.bbox.0, tfeat.bbox.1, tfeat.bbox.2, tfeat.bbox.3];
        let feat_geom_type_string = tfeat.geometry.type_;
        let geometry_dict = PyDict::new(py);
        geometry_dict.set_item("type", feat_geom_type_string)?;
        geometry_dict.set_item("coordinates", tfeat.geometry.coordinates)?;
        // let geometry_items = vec![
        //     ("type".to_string(),
        //
        //         PyString::new(py, feat_geom_type_string)
        //     ),
        //     (
        //         "coordinates".to_string(),
        //         tfeat.geometry.coordinates.to_object(py),
        //     ),
        // ]
        // .into_iter()
        // .collect::<HashMap<String, PyObject>>();
        // Create the properties dictionary
        // let mut properties_dict: HashMap<String, Py<PyAny>> = HashMap::new();
        let properties_dict = PyDict::new(py);
        let (x, y, z) = self.tuple();
        let xyz_tuple_string = format!("({x}, {y}, {z})");
        let title_string = format!("XYZ tile {xyz_tuple_string}");

        properties_dict.set_item("title", title_string)?;
        // properties_dict.insert(
        //     "title".to_string(),
        //     format!("XYZ tile {xyz_tuple_string}").into_py(py),
        // );
        if let Some(props) = props {
            for (k, v) in props.iter() {
                properties_dict.set_item(k, v)?;
            }
        }
        feature_dict.set_item(intern!(py, "type"), "Feature")?;

        feature_dict.set_item(intern!(py, "bbox"), bbox_vec)?;
        feature_dict.set_item(intern!(py, "id"), tfeat.id)?;
        feature_dict.set_item(intern!(py, "geometry"), geometry_dict)?;
        feature_dict.set_item("properties".to_string(), properties_dict)?;
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
    fn parent(&self, zoom: Option<u8>) -> Option<Self> {
        self.parent(zoom)
    }

    fn root() -> Self {
        pytile!(0, 0, 0)
    }
}

impl TileChildren1 for PyTile {
    fn children1(&self) -> [PyTile; 4] {
        self.children1()
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
