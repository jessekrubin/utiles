use hex::ToHex;
use md5::Digest;
use noncrypto_digests::Fnv;
use xxhash_rust::const_xxh3::xxh3_64 as const_xxh3;
use xxhash_rust::const_xxh3::xxh3_128 as const_xxh3_128;
use xxhash_rust::const_xxh32::xxh32 as const_xxh32;
use xxhash_rust::const_xxh64::xxh64 as const_xxh64;

#[inline]
pub fn md5_be(data: impl AsRef<[u8]>) -> [u8; 16] {
    let mut hasher = md5::Md5::new();
    hasher.update(data.as_ref());
    hasher.finalize().into()
}

#[inline]
pub fn fnv1a_be(data: impl AsRef<[u8]>) -> [u8; 8] {
    let mut hasher = Fnv::default();
    hasher.update(data.as_ref());
    hasher.finalize().into()
}

#[inline]
pub fn xxh32_be(data: impl AsRef<[u8]>) -> [u8; 4] {
    const_xxh32(data.as_ref(), 0).to_be_bytes()
}

#[inline]
pub fn xxh64_be(data: impl AsRef<[u8]>) -> [u8; 8] {
    const_xxh64(data.as_ref(), 0).to_be_bytes()
}

#[inline]
pub fn xxh3_be(data: impl AsRef<[u8]>) -> [u8; 8] {
    const_xxh3(data.as_ref()).to_be_bytes()
}

#[inline]
pub fn xxh3_128_be(data: impl AsRef<[u8]>) -> [u8; 16] {
    const_xxh3_128(data.as_ref()).to_be_bytes()
}

macro_rules! hex_fns {
    ($hash_fn:ident, $hex_name:ident, $hex_upper_name:ident, $output_size:expr ) => {
        #[inline]
        pub fn $hex_name(data: impl AsRef<[u8]>) -> String {
            $hash_fn(data).encode_hex::<String>()
        }

        #[inline]
        pub fn $hex_upper_name(data: impl AsRef<[u8]>) -> String {
            $hash_fn(data).encode_hex_upper::<String>()
        }
    };
}
hex_fns!(fnv1a_be, fnv1a_be_hex, fnv1a_be_hex_upper, 8);
hex_fns!(xxh32_be, xxh32_be_hex, xxh32_be_hex_upper, 4);
hex_fns!(xxh64_be, xxh64_be_hex, xxh64_be_hex_upper, 8);
hex_fns!(xxh3_be, xxh3_be_hex, xxh3_be_hex_upper, 8);
hex_fns!(xxh3_128_be, xxh3_128_be_hex, xxh3_128_be_hex_upper, 16);
