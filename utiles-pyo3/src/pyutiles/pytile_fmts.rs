//! py-tile-fmts module for wrapping the `utiles::TileStringFormatter`
use std::hash::Hash;

use pyo3::prelude::*;
use pyo3::types::PyTuple;

use utiles::TileStringFormatter;

use crate::pyutiles::pyparsing::parse_tile_arg;

#[derive(Clone, Debug, PartialEq, Hash)]
#[pyclass(name = "TileFmts", module = "utiles._utiles", frozen)]
pub struct PyTileFmts {
    pub tformatter: TileStringFormatter,
}

#[pymethods]
impl PyTileFmts {
    #[new]
    pub fn py_new(fmtstr: &str) -> Self {
        PyTileFmts {
            tformatter: TileStringFormatter::new(fmtstr),
        }
    }

    #[pyo3(signature = (* args))]
    pub fn fmt(&self, args: &Bound<'_, PyTuple>) -> PyResult<String> {
        let tile = parse_tile_arg(args)?;
        Ok(self.tformatter.fmt(&tile.xyz))
    }

    #[pyo3(signature = (* args))]
    pub fn format(&self, args: &Bound<'_, PyTuple>) -> PyResult<String> {
        let tile = parse_tile_arg(args)?;
        Ok(self.tformatter.fmt(&tile.xyz))
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
