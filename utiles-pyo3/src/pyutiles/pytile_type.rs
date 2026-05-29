use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};
use utiles::tile_type;
use utiles::tile_type::{TileEncoding, TileFormat, TileType};

#[pyclass(name = "TileType", module = "utiles._utiles")]
pub struct PyTileType(TileType);

const ENCODING_STRINGS: &str = "uncompressed, internal, zlib, gzip, brotli, zstd";

struct PyTileEncoding(TileEncoding);

impl<'py> FromPyObject<'_, 'py> for PyTileEncoding {
    type Error = PyErr;

    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        if let Ok(s) = ob.cast_exact::<PyString>() {
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
                "Invalid encoding (options: {ENCODING_STRINGS})"
            )))
        }
    }
}

struct PyTileFormat(TileFormat);

const TILE_FORMAT_STRINGS: &str = "png, webp, pbf, mvt, gif, jpg, jpeg, json, geojson";
impl<'py> FromPyObject<'_, 'py> for PyTileFormat {
    type Error = PyErr;
    fn extract(ob: Borrowed<'_, 'py, PyAny>) -> PyResult<Self> {
        if let Ok(s) = ob.cast_exact::<PyString>() {
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
                "Invalid format (options: {TILE_FORMAT_STRINGS})"
            )))
        }
    }
}

#[pymethods]
impl PyTileType {
    #[new]
    fn py_new(format: PyTileFormat, encoding: PyTileEncoding) -> Self {
        let encoding = encoding.0;
        let format = format.0;
        let ttype = TileType::new(format, encoding);
        Self(ttype)
    }

    fn __repr__(&self) -> String {
        format!(
            "TileType(format=\"{}\", encoding=\"{}\")",
            self.0.format, self.0.encoding
        )
    }

    #[getter]
    fn format<'py>(&self, py: Python<'py>) -> &Bound<'py, PyString> {
        match self.0.format {
            TileFormat::Png => pyo3::intern!(py, "png"),
            TileFormat::Jpg => pyo3::intern!(py, "jpg"),
            TileFormat::Gif => pyo3::intern!(py, "gif"),
            TileFormat::Webp => pyo3::intern!(py, "webp"),
            TileFormat::Pbf => pyo3::intern!(py, "pbf"),
            TileFormat::Mlt => pyo3::intern!(py, "mlt"),
            TileFormat::Json => pyo3::intern!(py, "json"),
            TileFormat::GeoJson => pyo3::intern!(py, "geojson"),
            TileFormat::Tiff => pyo3::intern!(py, "tiff"),
            TileFormat::Unknown => pyo3::intern!(py, "unknown"),
        }
    }

    #[getter]
    fn encoding<'py>(&self, py: Python<'py>) -> &Bound<'py, PyString> {
        match self.0.encoding {
            TileEncoding::Uncompressed => pyo3::intern!(py, "uncompressed"),
            TileEncoding::Internal => pyo3::intern!(py, "internal"),
            TileEncoding::Zlib => pyo3::intern!(py, "zlib"),
            TileEncoding::Gzip => pyo3::intern!(py, "gzip"),
            TileEncoding::Brotli => pyo3::intern!(py, "brotli"),
            TileEncoding::Zstd => pyo3::intern!(py, "zstd"),
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
pub(crate) fn tiletype(buf: &[u8]) -> PyTileType {
    let ttype = tile_type::tiletype(buf);
    PyTileType(ttype)
}

#[pyfunction]
pub(crate) fn tiletype_str(buf: &[u8]) -> String {
    tile_type::tiletype_str(buf)
}

#[pyfunction]
pub(crate) fn tiletype2headers(tiletype: usize) -> Vec<(&'static str, &'static str)> {
    tile_type::headers(&tile_type::const2enum(tiletype))
}
