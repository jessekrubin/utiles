use pyo3::prelude::PyAnyMethods;
use pyo3::types::{PyString, PyType};
use pyo3::{
    Bound, FromPyObject, PyAny, PyErr, PyResult, Python, intern, pyclass, pyfunction,
    pymethods,
};
use utiles::tile_type;
use utiles::tile_type::{TileEncoding, TileFormat, TileType};

#[pyclass(name = "TileType", module = "utiles._utiles")]
pub struct PyTileType(TileType);

const ENCODING_STRINGS: &str = "uncompressed, internal, zlib, gzip, brotli, zstd";

pub struct PyTileEncoding(TileEncoding);

impl FromPyObject<'_> for PyTileEncoding {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(s) = ob.downcast::<PyString>() {
            let s = s.to_string().to_ascii_lowercase();
            match s.as_str() {
                "uncompressed" => Ok(Self(TileEncoding::Uncompressed)),
                "internal" => Ok(Self(TileEncoding::Internal)),
                "zlib" => Ok(Self(TileEncoding::Zlib)),
                "gzip" => Ok(Self(TileEncoding::Gzip)),
                "brotli" => Ok(Self(TileEncoding::Brotli)),
                "zstd" => Ok(Self(TileEncoding::Zstd)),
                _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid TileEncoding: {s} (options: {ENCODING_STRINGS})"
                ))),
            }
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "Invalid encoding: {ob} (options: {ENCODING_STRINGS})"
            )))
        }
    }
}

pub struct PyTileFormat(TileFormat);

const TILE_FORMAT_STRINGS: &str = "png, webp, pbf, mvt, gif, jpg, jpeg, json, geojson";
impl FromPyObject<'_> for PyTileFormat {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(s) = ob.downcast::<PyString>() {
            let f_str = s.to_string();

            let tf: Option<TileFormat> = TileFormat::try_parse(&f_str);
            match tf {
                Some(f) => Ok(Self(f)),
                None => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid TileFormat: {f_str} (options: {TILE_FORMAT_STRINGS})"
                ))),
            }
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "Invalid encoding: {ob} (options: {TILE_FORMAT_STRINGS})"
            )))
        }
    }
}

#[pymethods]
impl PyTileType {
    #[new]
    pub fn py_new(format: PyTileFormat, encoding: PyTileEncoding) -> Self {
        let encoding = encoding.0;
        let format = format.0;
        let ttype = TileType::new(format, encoding);
        Self(ttype)
    }

    pub fn __repr__(&self) -> String {
        format!(
            "TileType(format=\"{}\", encoding=\"{}\")",
            self.0.format, self.0.encoding
        )
    }

    #[getter]
    fn format<'py>(&self, py: Python<'py>) -> &Bound<'py, PyString> {
        match self.0.format {
            TileFormat::Png => intern!(py, "png"),
            TileFormat::Jpg => intern!(py, "jpg"),
            TileFormat::Gif => intern!(py, "gif"),
            TileFormat::Webp => intern!(py, "webp"),
            TileFormat::Pbf => intern!(py, "pbf"),
            TileFormat::Mlt => intern!(py, "mlt"),
            TileFormat::Json => intern!(py, "json"),
            TileFormat::GeoJson => intern!(py, "geojson"),
            TileFormat::Tiff => intern!(py, "tiff"),
            TileFormat::Unknown => intern!(py, "unknown"),
        }
    }

    #[getter]
    fn encoding<'py>(&self, py: Python<'py>) -> &Bound<'py, PyString> {
        match self.0.encoding {
            TileEncoding::Uncompressed => intern!(py, "uncompressed"),
            TileEncoding::Internal => intern!(py, "internal"),
            TileEncoding::Zlib => intern!(py, "zlib"),
            TileEncoding::Gzip => intern!(py, "gzip"),
            TileEncoding::Brotli => intern!(py, "brotli"),
            TileEncoding::Zstd => intern!(py, "zstd"),
        }
    }

    #[getter]
    fn compression<'py>(&self, py: Python<'py>) -> &Bound<'py, PyString> {
        self.encoding(py)
    }

    #[getter]
    fn headers(&self) -> Vec<(&'static str, &'static str)> {
        self.0.headers_vec()
    }

    #[classmethod]
    fn from_bytes(_cls: &Bound<'_, PyType>, buffer: &[u8]) -> Self {
        Self(tile_type::tiletype(buffer))
    }

    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[pyfunction]
pub(crate) fn tiletype(buffer: &[u8]) -> PyTileType {
    let ttype = tile_type::tiletype(buffer);
    PyTileType(ttype)
}

#[pyfunction]
pub(crate) fn tiletype_str(buffer: &[u8]) -> String {
    tile_type::tiletype_str(buffer)
}

#[pyfunction]
pub(crate) fn tiletype2headers(tiletype: usize) -> Vec<(&'static str, &'static str)> {
    tile_type::headers(&tile_type::const2enum(tiletype))
}
