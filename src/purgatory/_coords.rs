/// def pycoords(obj):
///     if isinstance(obj, (tuple, list)):
///         coordinates = obj
///     elif "features" in obj:
///         coordinates = [feat["geometry"]["coordinates"] for feat in obj["features"]]
///     elif "geometry" in obj:
///         coordinates = obj["geometry"]["coordinates"]
///     else:
///         coordinates = obj.get("coordinates", obj)
///     for e in coordinates:
///         if isinstance(e, (float, int)):
///             return tuple(coordinates)
///         else:
///             for f in pycoords(e):
// #[pyfunction]
// fn _coords_slow(py: Python, obj: &PyAny) -> PyResult<Vec<(f64, f64)>> {
//     println!("obj: {:?}", obj);
//     // let is_tuple = obj.is_instance(PyTuple)?;
//     let tuple_type = py.get_type::<PyTuple>();
//     let list_type = py.get_type::<PyList>();
//     let float_type = py.get_type::<PyFloat>();
//     let int_type = py.get_type::<PyInt>();
//     let dict_type = py.get_type::<PyDict>();
//
//     let is_tuple = obj.is_instance(tuple_type)?;
//     let is_list = obj.is_instance(list_type)?;
//     let is_dict = obj.is_instance(dict_type)?;
//
//     let mut result: Vec<(f64, f64)> = Vec::new();
//     let coordinates: &PyAny;
//     if is_tuple {
//         println!("is tuple");
//         // let tuple_obj = obj.into_object(py);
//         // let tuple_obj = tuple_obj.downcast::<PyTuple>()?;
//         // coordinates = obj.into();
//         // coordinates = obj;
//         // println!("tuple_obj: {:?}", coordinates);
//         let tuple_len = obj.len();
//         if let Ok(tlen) = tuple_len {
//             if tlen == 2 {
//                 // try to extract as coords
//                 return Ok(vec![(
//                     obj.get_item(0)?.extract::<f64>()?,
//                     obj.get_item(1)?.extract::<f64>()?,
//                 )]);
//             } else if tlen == 3 {
//                 // try to extract as coords
//                 // if  first value is a number assume the thing is a coord
//                 if obj.get_item(0)?.is_instance(float_type)?
//                     || obj.get_item(0)?.is_instance(int_type)?
//                 {
//                     return Ok(vec![(
//                         obj.get_item(0)?.extract::<f64>()?,
//                         obj.get_item(1)?.extract::<f64>()?,
//                     )]);
//                 } else {
//                     // call _coords on each item
//                     let mut coordsvec: Vec<(f64, f64)> = Vec::new();
//                     for item in obj.iter() {
//                         let c = _coords_slow(py, item)?;
//                         coordsvec.extend(c);
//                     }
//                     return Ok(coordsvec);
//                 }
//                 // return Ok(vec![(obj.get_item(0)?.extract::<f64>()?, obj.get_item(1)?.extract::<f64>()?)]);
//             }
//         }
//
//         // let c = obj.extract
//     } else if is_list {
//         println!("is list");
//         // let tuple_obj = obj.into_object(py);
//         // let tuple_obj = tuple_obj.downcast::<PyTuple>()?;
//         // coordinates = obj.into();
//         coordinates = obj;
//         let c = obj.downcast::<PyList>()?;
//         for item in c.iter() {
//             if item.is_instance(float_type)? || item.is_instance(int_type)? {
//                 println!("item: {:?}", item);
//                 let c = _coords_slow(py, item)?;
//                 println!("c: {:?}", c);
//                 let value = item.extract::<f64>()?;
//                 result.push((value, value));
//             } else {
//                 println!("item: {:?}", item);
//                 let c = _coords_slow(py, item)?;
//                 println!("c: {:?}", c);
//                 result.extend(c);
//             }
//         }
//         println!("tuple_obj: {:?}", coordinates);
//     } else if is_dict {
//         println!("is dict");
//         match obj.contains("features") {
//             Ok(true) => {
//                 let features = obj.get_item("features");
//
//                 match features {
//                     Ok(val) => {
//                         println!("val: {:?}", val);
//                         let c = _coords_slow(py, val)?;
//                         println!("c: {:?}", c);
//                         result.extend(c);
//                     }
//                     Err(e) => {
//                         println!("e: {:?}", e);
//                     }
//                 }
//             }
//             _ => {
//                 println!("is not features");
//             }
//         }
//
//         match obj.contains("geometry") {
//             Ok(true) => {
//                 let features = obj.get_item("geometry");
//
//                 match features {
//                     Ok(val) => {
//                         println!("val: {:?}", val);
//                         let c = _coords_slow(py, val)?;
//                         println!("c: {:?}", c);
//                         result.extend(c);
//                     }
//                     Err(e) => {
//                         println!("e: {:?}", e);
//                     }
//                 }
//             }
//             _ => {
//                 println!("is not geometry");
//             }
//         }
//
//         match obj.contains("coordinates") {
//             Ok(true) => {
//                 let features = obj.get_item("geometry");
//
//                 match features {
//                     Ok(val) => {
//                         println!("val: {:?}", val);
//                         let c = _coords_slow(py, val)?;
//                         println!("c: {:?}", c);
//                         result.extend(c);
//                     }
//                     Err(e) => {
//                         println!("e: {:?}", e);
//                     }
//                 }
//             }
//             _ => {
//                 println!("is not geometry");
//             }
//         }
//         // println!("features: {:?}", features);
//     } else {
//         println!("is something else");
//         // dummy obj
//         // coordinates = obj.getattr("coordinates")?.into();
//     }
//
//     // Err
//     // raise dummy error
//     // Err(PyErr::new::<PyValueError, _>( "the tile argument may have 1 or 4 values. Note that zoom is a keyword-only argument"))?;
//     Ok(result)
// }
