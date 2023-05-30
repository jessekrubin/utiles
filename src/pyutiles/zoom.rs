use pyo3::FromPyObject;

#[derive(FromPyObject)]
pub enum ZoomsOrInt {
    Zooms(Vec<u8>),
    Int(u8),
}
