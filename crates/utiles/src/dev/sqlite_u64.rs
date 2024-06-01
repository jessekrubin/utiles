#[must_use]
pub fn u64_to_i64_shift(value: u64) -> i64 {
    // The constant 9,223,372,036,854,775,807 is the maximum value for i64
    const OFFSET: u64 = 9_223_372_036_854_775_808;

    // Subtracting the OFFSET from the u64 value. This might wrap around, which is expected.
    // Wrapping sub is used to explicitly indicate that wrap around is intended.

    value.wrapping_sub(OFFSET) as i64
}

#[must_use]
pub fn i64_to_u64_shift(value: i64) -> u64 {
    // The constant 9,223,372,036,854,775,807 is the maximum value for i64
    const OFFSET: u64 = 9_223_372_036_854_775_808;

    // Adding the OFFSET back to the i64 value to get the original u64 value.
    // This is safe because we're adding to an i64, which will not overflow an u64.

    (value as u64).wrapping_add(OFFSET)
}

#[must_use]
pub fn u64_to_i64_unsafe_transmute(value: u64) -> i64 {
    unsafe { std::mem::transmute(value) }
}

#[must_use]
pub fn i64_to_u64_unsafe_transmute(value: i64) -> u64 {
    unsafe { std::mem::transmute(value) }
}

#[must_use]
pub fn u64_to_i64_ptr(value: u64) -> i64 {
    let ptr = std::ptr::addr_of!(value).cast::<i64>();
    unsafe { *ptr }
}

#[must_use]
pub fn i64_to_u64_ptr(value: i64) -> u64 {
    let ptr = std::ptr::addr_of!(value).cast::<u64>();
    unsafe { *ptr }
}

#[must_use]
pub fn u64_to_i64_ne_bytes(value: u64) -> i64 {
    i64::from_ne_bytes(value.to_ne_bytes())
}

#[must_use]
pub fn i64_to_u64_ne_bytes(value: i64) -> u64 {
    u64::from_ne_bytes(value.to_ne_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_conversion {
        ($name:ident, $u64_to_i64:expr, $i64_to_u64:expr, $values:expr) => {
            #[test]
            fn $name() {
                let u64_to_i64 = $u64_to_i64;
                let i64_to_u64 = $i64_to_u64;
                for &(original_u64, expected_i64) in $values.iter() {
                    assert_eq!(
                        u64_to_i64(original_u64),
                        expected_i64,
                        "Failed on value: {}",
                        original_u64
                    );
                    assert_eq!(
                        i64_to_u64(expected_i64),
                        original_u64,
                        "Failed on value: {}",
                        expected_i64
                    );
                }
            }
        };
    }

    const U64_I64_VALUES: &[(u64, i64)] = &[
        (0, 0),
        (0xFFFF_FFFF_FFFF_FFFF, -1),
        (9_223_372_036_854_775_807 + 1234, -9_223_372_036_854_774_575),
        (1, 1),
    ];

    test_conversion!(
        test_u64_to_i64_transmute,
        u64_to_i64_unsafe_transmute,
        i64_to_u64_unsafe_transmute,
        U64_I64_VALUES
    );
    test_conversion!(
        test_u64_to_i64_ptr,
        u64_to_i64_ptr,
        i64_to_u64_ptr,
        U64_I64_VALUES
    );
    test_conversion!(
        test_u64_to_i64_ne_bytes,
        u64_to_i64_ne_bytes,
        i64_to_u64_ne_bytes,
        U64_I64_VALUES
    );
}

#[cfg(test)]
mod i64_shift_tests {
    #![allow(clippy::similar_names)]

    use super::*;

    #[test]
    fn test_u64_to_i64_left() {
        let original: u64 = 0;
        let expected: i64 = -9_223_372_036_854_775_808;
        assert_eq!(u64_to_i64_shift(original), expected);
        let back: u64 = i64_to_u64_shift(expected);
        assert_eq!(back, original);
    }

    #[test]
    fn test_i64_to_u64_middle() {
        let original: i64 = 0;
        let expected: u64 = 9_223_372_036_854_775_808;
        assert_eq!(i64_to_u64_shift(original), expected);
        let back: i64 = u64_to_i64_shift(expected);
        assert_eq!(back, original);
    }

    #[test]
    fn test_u64_to_i64_right() {
        let original: u64 = 0xFFFF_FFFF_FFFF_FFFF; // Max u64 value
        let expected: i64 = 9_223_372_036_854_775_807;
        assert_eq!(u64_to_i64_shift(original), expected);
        let back: u64 = i64_to_u64_shift(expected);
        assert_eq!(back, original);
    }

    #[test]
    fn test_round_trip() {
        // Tests that converting back and forth yields the original value
        let original_u64: u64 = 0xFEDC_BA98_7654_3210;
        let intermediate_i64: i64 = u64_to_i64_shift(original_u64);
        let final_u64: u64 = i64_to_u64_shift(intermediate_i64);
        assert_eq!(original_u64, final_u64);

        let original_i64: i64 = -0x0123_4567_89AB_CDEF;
        let intermediate_u64: u64 = i64_to_u64_shift(original_i64);
        let final_i64: i64 = u64_to_i64_shift(intermediate_u64);
        assert_eq!(original_i64, final_i64);
    }
}
