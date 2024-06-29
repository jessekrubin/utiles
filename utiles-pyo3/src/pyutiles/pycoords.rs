use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::{exceptions, pyfunction, Bound, FromPyObject, PyAny, PyErr, PyResult};

use crate::pyutiles::pyiters::CoordinateIterator;

#[derive(FromPyObject, Debug)]
pub enum CoordsExtractor<'a> {
    ListVecF64(Vec<Vec<f64>>),
    VecF64(Vec<f64>),
    IntTuple3d((i32, i32, i32)),
    IntTuple2d((i32, i32)),
    List(Vec<Bound<'a, PyAny>>),
    Tuple(Vec<Bound<'a, PyAny>>),
    Dict(HashMap<String, Bound<'a, PyAny>>),
    #[pyo3(transparent)]
    CatchAll(Bound<'a, PyAny>), // This extraction never fails
}

#[pyfunction]
pub fn _coords(obj: &Bound<'_, PyAny>) -> PyResult<CoordinateIterator> {
    let thing = CoordsExtractor::extract_bound(obj)?;
    match thing {
        CoordsExtractor::ListVecF64(v) => {
            // ensure 2d
            let iter = v.into_iter().map(|t| (t[0], t[1]));
            Ok(CoordinateIterator {
                iter: Box::new(iter.into_iter()),
            })
        }
        CoordsExtractor::VecF64(v) => {
            // ensure 2d
            let vec = vec![(v[0], v[1])];
            Ok(CoordinateIterator {
                iter: Box::new(vec.into_iter()),
            })
        }
        CoordsExtractor::IntTuple3d(t) => {
            let iter = vec![(f64::from(t.0), f64::from(t.1))];
            Ok(CoordinateIterator {
                iter: Box::new(iter.into_iter()),
            })
        }
        CoordsExtractor::IntTuple2d(t) => {
            // return an iterator of the tuple
            let iter = vec![(f64::from(t.0), f64::from(t.1))];
            Ok(CoordinateIterator {
                iter: Box::new(iter.into_iter()),
            })
        }
        CoordsExtractor::List(l) => {
            if l.len() == 2 {
                // try to extract as coords
                return Ok(CoordinateIterator {
                    iter: Box::new(
                        vec![(l[0].extract::<f64>()?, l[1].extract::<f64>()?)]
                            .into_iter(),
                    ),
                });
            }
            let mut coordsvec: Vec<(f64, f64)> = Vec::new();
            for item in l {
                let c = _coords(&item.as_borrowed())?;
                let cv = c.iter.collect::<Vec<_>>();
                coordsvec.extend(cv);
            }
            Ok(CoordinateIterator {
                iter: Box::new(coordsvec.into_iter()),
            })
        }
        CoordsExtractor::Tuple(t) => {
            if t.is_empty() {
                return Ok(CoordinateIterator {
                    iter: Box::new(vec![].into_iter()),
                });
            }
            if t.len() == 1 {
                let res = _coords(&t[0].as_borrowed());
                return res;
            }
            Ok(CoordinateIterator {
                iter: Box::new(vec![].into_iter()),
            })
        }
        CoordsExtractor::Dict(d) => {
            // extract the sub dict key 'coordinates'
            if let Some(coords) = d.get("coordinates") {
                let c = coords;
                let res = _coords(&c.as_borrowed());
                return res;
            }
            // extract the sub dict
            if let Some(geom) = d.get("geometry") {
                // recurse around again
                let geom_ref = geom;
                let res = _coords(&geom_ref.as_borrowed());
                return Ok(res.unwrap());
            }
            if let Some(features) = d.get("features") {
                if let Ok(features) = features.extract::<Vec<Bound<PyDict>>>() {
                    // chain the iterators
                    let mut coords = vec![];
                    for feature in features {
                        let res = _coords(&feature.as_borrowed())?;
                        coords.extend(res.iter);
                    }
                    return Ok(CoordinateIterator {
                        iter: Box::new(coords.into_iter()),
                    });
                }
                // return empty iterator
                return Ok(CoordinateIterator {
                    iter: Box::new(vec![].into_iter()),
                });
            }
            // return empty iterator
            Ok(CoordinateIterator {
                iter: Box::new(vec![].into_iter()),
            })
        }
        CoordsExtractor::CatchAll(_c) => {
            Err(PyErr::new::<exceptions::PyTypeError, _>("NO COORDS"))
        }
    }
}

#[pyfunction]
pub fn coords(obj: &Bound<'_, PyAny>) -> PyResult<Vec<(f64, f64)>> {
    Ok(_coords(obj)?.iter.collect())
}
