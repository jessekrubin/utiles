//! py-tile-fmts module for wrapping the `utiles::TileStringFormatter`
use std::hash::Hash;

use pyo3::prelude::*;
use utiles::TileStringFormatter;

use crate::pyutiles::pyparsing::PyTileArg;

#[derive(Clone, Debug, PartialEq, Hash)]
#[pyclass(
    name = "TileFmts",
    module = "utiles._utiles",
    frozen,
    skip_from_py_object
)]
pub struct PyTileFmts(TileStringFormatter);

#[pymethods]
impl PyTileFmts {
    #[new]
    fn py_new(fmtstr: &str) -> Self {
        Self(TileStringFormatter::new(fmtstr))
    }

    #[pyo3(signature = (* args))]
    fn fmt(&self, args: PyTileArg) -> String {
        self.0.fmt(&args)
    }

    #[pyo3(signature = (* args))]
    fn format(&self, args: PyTileArg) -> String {
        self.0.fmt(&args)
    }

    #[getter]
    fn fmtstr(&self) -> &str {
        self.0.fmtstr()
    }

    fn __str__(&self) -> String {
        format!("TileFmts({})", self.0.fmtstr())
    }

    fn __repr__(&self) -> String {
        format!(
            "TileFmts({:?}) # tokens ({:?}): {:?}",
            self.0.fmtstr(),
            self.0.tokens().len(),
            self.0.tokens(),
        )
    }
}
