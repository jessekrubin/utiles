pub trait BoundingBoxLike {
    fn west(&self) -> f64;
    fn south(&self) -> f64;
    fn east(&self) -> f64;
    fn north(&self) -> f64;

    fn left(&self) -> f64;
    fn bottom(&self) -> f64;
    fn right(&self) -> f64;
    fn top(&self) -> f64;
}

pub trait TLngLat {
    fn lng(&self) -> f64;
    fn lat(&self) -> f64;
}

pub trait TGeoBbox {
    fn north(&self) -> f64;
    fn south(&self) -> f64;
    fn east(&self) -> f64;
    fn west(&self) -> f64;
}

pub trait Utiles<TLngLat, TGeoBbox> {
    fn ul(&self) -> TLngLat;
    fn ur(&self) -> TLngLat;
    fn lr(&self) -> TLngLat;
    fn ll(&self) -> TLngLat;
    fn bbox(&self) -> TGeoBbox;
}
