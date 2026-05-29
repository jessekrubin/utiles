use std::ops::Deref;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use utiles::TileLike;

use crate::pyutiles::pylnglatbbox::PyLngLatBbox;
use crate::pyutiles::pytile::PyTile;

#[derive(Debug)]
pub enum PyTileArg<'a, 'py> {
    PyTile(Borrowed<'a, 'py, PyTile>),
    PyTileLike(PyTile),
}

impl TileLike for PyTileArg<'_, '_> {
    fn x(&self) -> u32 {
        match self {
            PyTileArg::PyTile(tile) => tile.get().x(),
            PyTileArg::PyTileLike(tile) => tile.x(),
        }
    }

    fn y(&self) -> u32 {
        match self {
            PyTileArg::PyTile(tile) => tile.get().y(),
            PyTileArg::PyTileLike(tile) => tile.y(),
        }
    }

    fn z(&self) -> u8 {
        match self {
            PyTileArg::PyTile(tile) => tile.get().z(),
            PyTileArg::PyTileLike(tile) => tile.z(),
        }
    }
}

impl Deref for PyTileArg<'_, '_> {
    type Target = PyTile;

    fn deref(&self) -> &Self::Target {
        match self {
            PyTileArg::PyTile(tile) => tile.get(),
            PyTileArg::PyTileLike(tile) => tile,
        }
    }
}

impl From<PyTile> for PyTileArg<'_, '_> {
    fn from(tile: PyTile) -> Self {
        PyTileArg::PyTileLike(tile)
    }
}

impl From<PyTileArg<'_, '_>> for PyTile {
    fn from(arg: PyTileArg) -> Self {
        match arg {
            PyTileArg::PyTile(tile) => *tile.get(),
            PyTileArg::PyTileLike(tile) => tile,
        }
    }
}

impl<'a, 'py> From<Borrowed<'a, 'py, PyTile>> for PyTileArg<'a, 'py> {
    fn from(tile: Borrowed<'a, 'py, PyTile>) -> Self {
        PyTileArg::PyTile(tile)
    }
}

impl From<(u32, u32, u8)> for PyTileArg<'_, '_> {
    fn from(xyz: (u32, u32, u8)) -> Self {
        PyTileArg::from(PyTile::py_new(xyz.0, xyz.1, xyz.2))
    }
}

// impl FromPyObject
impl<'a, 'py> FromPyObject<'a, 'py> for PyTileArg<'a, 'py> {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if let Ok(tup) = obj.cast_exact::<PyTuple>() {
            match tup.len() {
                1 => {
                    let item = tup.get_item(0)?;
                    if let Ok(tile) = item.extract::<PyTile>() {
                        Ok(PyTileArg::from(tile))
                    } else if let Ok(seq) = item.extract::<(u32, u32, u8)>() {
                        Ok(PyTileArg::from(seq))
                    } else if let Ok(seq) = item.extract::<Vec<u32>>() {
                        Ok(PyTileArg::from((seq[0], seq[1], seq[2] as u8)))
                    } else {
                        Err(PyErr::new::<PyValueError, _>(
                            "the tile argument may have 1 or 3 values. Note that zoom is a keyword-only argument",
                        ))?
                    }
                }
                3 => tup.extract::<(u32, u32, u8)>().map(Self::from),
                _ => Err(PyErr::new::<PyValueError, _>(
                    "the tile argument may have 1 or 3 values. Note that zoom is a keyword-only argument",
                ))?,
            }
        } else if let Ok(pt) = obj.cast_exact::<PyTile>() {
            Ok(PyTileArg::from(pt))
        } else {
            Err(PyErr::new::<PyValueError, _>(
                "the tile argument may have 1 or 3 values. Note that zoom is a keyword-only argument",
            ))?
        }
    }
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub(crate) fn parse_tile_arg(args: PyTileArg) -> PyTile {
    PyTile::from(args)
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub(crate) fn parse_bbox(args: &Bound<'_, PyTuple>) -> PyResult<PyLngLatBbox> {
    let arglen = args.len();
    match arglen {
        1 => {
            let arg = args.get_item(0)?;
            if let Ok(bbox) = arg.extract::<(f64, f64, f64, f64)>() {
                return Ok(PyLngLatBbox::py_new(bbox.0, bbox.1, bbox.2, bbox.3));
            } else if let Ok(seq) = arg.extract::<Vec<f64>>() {
                return Ok(PyLngLatBbox::py_new(seq[0], seq[1], seq[2], seq[3]));
            }
            Err(PyErr::new::<PyValueError, _>(
                "the bbox argument may have 1, 2 or 4 values",
            ))
        }
        2 => {
            let x = args.get_item(0)?.extract()?;
            let y = args.get_item(1)?.extract()?;
            Ok(PyLngLatBbox::py_new(x, y, x, y))
        }
        4 => {
            let x1 = args.get_item(0)?.extract()?;
            let y1 = args.get_item(1)?.extract()?;
            let x2 = args.get_item(2)?.extract()?;
            let y2 = args.get_item(3)?.extract()?;
            Ok(PyLngLatBbox::py_new(x1, y1, x2, y2))
        }
        _ => Err(PyErr::new::<PyValueError, _>(
            "the bbox argument may have 1, 2 or 4 values",
        ))?,
    }
}

#[pyfunction]
#[pyo3(signature = (*args))]
pub(crate) fn _parse_tile_arg(args: PyTileArg) -> PyTile {
    parse_tile_arg(args)
}

#[pyfunction]
#[pyo3(signature = (*args))]
pub(crate) fn parse_tiles(args: &Bound<'_, PyTuple>) -> PyResult<Vec<PyTile>> {
    let nargs = args.len();
    if nargs == 1 {
        return crate::pyutiles::_extract(&args.get_item(0)?);
    } else if nargs == 3 {
        // if the first value is a number assume the thing is a tile
        if let Ok(t) = args.extract::<PyTileArg>() {
            Ok(vec![PyTile::from(t)])
        } else {
            let uno = args
                .get_item(0)
                .map(|item| item.extract::<PyTileArg>().map(PyTile::from))??;
            let dos = args
                .get_item(1)
                .map(|item| item.extract::<PyTileArg>().map(PyTile::from))??;
            let tres = args
                .get_item(2)
                .map(|item| item.extract::<PyTileArg>().map(PyTile::from))??;
            Ok(vec![uno, dos, tres])
        }
    } else {
        let mut tiles = Vec::with_capacity(nargs);
        for el in args.iter_borrowed() {
            let tile = el.extract::<PyTileArg>().map(PyTile::from)?;
            tiles.push(tile);
        }
        Ok(tiles)
    }
}

#[pyfunction]
pub(crate) fn parse_textiles(string: &str) -> Vec<PyTile> {
    utiles::parse_textiles(string)
        .into_iter()
        .map(PyTile::from)
        .collect()
}
