use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::{pyfunction, Bound, PyErr, PyResult};

use crate::pyutiles::pylnglatbbox::PyLngLatBbox;
use crate::pyutiles::pytile::PyTile;

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn parse_tile_arg(args: &Bound<'_, PyTuple>) -> PyResult<PyTile> {
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
pub fn parse_bbox(args: &Bound<'_, PyTuple>) -> PyResult<PyLngLatBbox> {
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
#[pyfunction]
#[pyo3(signature = (* args))]
pub fn _parse_tile_arg(args: &Bound<'_, PyTuple>) -> PyResult<PyTile> {
    parse_tile_arg(args)
}

#[pyfunction]
#[pyo3(signature = (* args))]
pub fn parse_tiles(args: &Bound<'_, PyTuple>) -> PyResult<Vec<PyTile>> {
    if args.len() == 1 {
        return crate::pyutiles::_extract(&args.get_item(0)?);
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
