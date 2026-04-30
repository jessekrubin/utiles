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
pub struct PyTileFmts {
    pub tformatter: TileStringFormatter,
}

#[pymethods]
impl PyTileFmts {
    #[new]
    pub fn py_new(fmtstr: &str) -> Self {
        Self {
            tformatter: TileStringFormatter::new(fmtstr),
        }
    }

    #[pyo3(signature = (* args))]
    pub fn fmt(&self, args: PyTileArg) -> String {
        self.tformatter.fmt(&args)
    }

    #[pyo3(signature = (* args))]
    pub fn format(&self, args: PyTileArg) -> String {
        self.tformatter.fmt(&args)
    }

    #[getter]
    pub fn fmtstr(&self) -> &str {
        self.tformatter.fmtstr()
    }

    pub fn __str__(&self) -> String {
        format!("TileFmts({})", self.tformatter.fmtstr())
    }

    pub fn __repr__(&self) -> String {
        format!(
            "TileFmts({:?}) # tokens ({:?}): {:?}",
            self.tformatter.fmtstr(),
            self.tformatter.tokens().len(),
            self.tformatter.tokens(),
        )
    }
}
