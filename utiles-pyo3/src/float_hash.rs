use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Float64Hash(f64);
const CANONICAL_F64_NAN_BITS: u64 = 0x7ff8_0000_0000_0000;

impl Hash for Float64Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bits = if self.0.is_nan() {
            CANONICAL_F64_NAN_BITS
        } else {
            self.0.to_bits()
        };
        bits.hash(state);
    }
}

impl From<f64> for Float64Hash {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
