pub fn u64_to_i64_shift(value: u64) -> i64 {
    // The constant 9,223,372,036,854,775,807 is the maximum value for i64
    const OFFSET: u64 = 9_223_372_036_854_775_808;

    // Subtracting the OFFSET from the u64 value. This might wrap around, which is expected.
    // Wrapping sub is used to explicitly indicate that wrap around is intended.

    value.wrapping_sub(OFFSET) as i64
}

pub fn i64_to_u64_shift(value: i64) -> u64 {
    // The constant 9,223,372,036,854,775,807 is the maximum value for i64
    const OFFSET: u64 = 9_223_372_036_854_775_808;

    // Adding the OFFSET back to the i64 value to get the original u64 value.
    // This is safe because we're adding to an i64, which will not overflow an u64.

    (value as u64).wrapping_add(OFFSET)
}

#[cfg(test)]
mod i64_shift_tests {
    use super::*;

    #[test]
    fn test_u64_to_i64_left() {
        let original: u64 = 0;
        let expected: i64 = -9223372036854775808;
        assert_eq!(u64_to_i64_shift(original), expected);
        let back: u64 = i64_to_u64_shift(expected);
        assert_eq!(back, original);
    }

    #[test]
    fn test_i64_to_u64_middle() {
        let original: i64 = 0;
        let expected: u64 = 9223372036854775808;
        assert_eq!(i64_to_u64_shift(original), expected);
        let back: i64 = u64_to_i64_shift(expected);
        assert_eq!(back, original);
    }

    #[test]
    fn test_u64_to_i64_right() {
        let original: u64 = 0xFFFFFFFFFFFFFFFF; // Max u64 value
        let expected: i64 = 9223372036854775807;
        assert_eq!(u64_to_i64_shift(original), expected);
        let back: u64 = i64_to_u64_shift(expected);
        assert_eq!(back, original);
    }

    #[test]
    fn test_round_trip() {
        // Tests that converting back and forth yields the original value
        let original_u64: u64 = 0xFEDCBA9876543210;
        let intermediate_i64: i64 = u64_to_i64_shift(original_u64);
        let final_u64: u64 = i64_to_u64_shift(intermediate_i64);
        assert_eq!(original_u64, final_u64);

        let original_i64: i64 = -0x123456789ABCDEF;
        let intermediate_u64: u64 = i64_to_u64_shift(original_i64);
        let final_i64: i64 = u64_to_i64_shift(intermediate_u64);
        assert_eq!(original_i64, final_i64);
    }

    // #[test]
    // fn test_i64_to_u64_zero() {
    //     let original: i64 = 0;
    //     let expected: u64 = 0;
    //     assert_eq!(i64_to_u64(original), expected);
    // }
    // #[test]
    // fn test_u64_to_i64_positive() {
    //     let original: u64 = 0x7FFFFFFFFFFFFFFF; // Max positive i64 value
    //     let expected: i64 = 0x7FFFFFFFFFFFFFFF; // Same value as i64
    //     assert_eq!(u64_to_i64(original), expected);
    // }
    //
    // #[test]
    // fn test_u64_to_i64_negative() {
    //     let original: u64 = 0xFFFFFFFFFFFFFFFF; // Max u64 value, interpreted as -1 in i64
    //     let expected: i64 = -1;
    //     assert_eq!(u64_to_i64(original), expected);
    // }
    //
    // #[test]
    // fn test_i64_to_u64_positive() {
    //     let original: i64 = 0x7FFFFFFFFFFFFFFF; // Max positive i64 value
    //     let expected: u64 = 0x7FFFFFFFFFFFFFFF; // Same value as u64
    //     assert_eq!(i64_to_u64(original), expected);
    // }
    //
    // #[test]
    // fn test_i64_to_u64_negative() {
    //     let original: i64 = -1;
    //     let expected: u64 = 0xFFFFFFFFFFFFFFFF; // Max u64 value
    //     assert_eq!(i64_to_u64(original), expected);
    // }
}

fn u64_to_i64_unsafe_transmute(value: u64) -> i64 {
    unsafe { std::mem::transmute(value) }
}

fn i64_to_u64_unsafe_transmute(value: i64) -> u64 {
    unsafe { std::mem::transmute(value) }
}

#[cfg(test)]
mod sqlite_u64_transmute_tests {
    use crate::utilesqlite::sqlite_u64::{
        i64_to_u64_unsafe_transmute, u64_to_i64_unsafe_transmute,
    };

    #[test]
    fn test_i64_to_u64_zero() {
        let original: i64 = 0;
        let expected: u64 = 0;
        assert_eq!(i64_to_u64_unsafe_transmute(original), expected);
    }

    #[test]
    fn test_u64_to_i64_positive() {
        let original: u64 = 0x7FFFFFFFFFFFFFFF; // Max positive i64 value
        let expected: i64 = 0x7FFFFFFFFFFFFFFF; // Same value as i64
        assert_eq!(u64_to_i64_unsafe_transmute(original), expected);
    }

    #[test]
    fn test_u64_to_i64_negative() {
        let original: u64 = 0xFFFFFFFFFFFFFFFF; // Max u64 value, interpreted as -1 in i64
        let expected: i64 = -1;
        assert_eq!(u64_to_i64_unsafe_transmute(original), expected);
    }

    #[test]
    fn test_i64_to_u64_positive() {
        let original: i64 = 0x7FFFFFFFFFFFFFFF; // Max positive i64 value
        let expected: u64 = 0x7FFFFFFFFFFFFFFF; // Same value as u64
        assert_eq!(i64_to_u64_unsafe_transmute(original), expected);
    }

    #[test]
    fn test_i64_to_u64_negative() {
        let original: i64 = -1;
        let expected: u64 = 0xFFFFFFFFFFFFFFFF; // Max u64 value
        assert_eq!(i64_to_u64_unsafe_transmute(original), expected);
    }
}

fn u64_to_i64_ptr(value: u64) -> i64 {
    let ptr = &value as *const u64 as *const i64;
    unsafe { *ptr }
}

fn i64_to_u64_ptr(value: i64) -> u64 {
    let ptr = &value as *const i64 as *const u64;
    unsafe { *ptr }
}

#[cfg(test)]
mod sqlite_u64_ptr_tests {
    use crate::utilesqlite::sqlite_u64::{i64_to_u64_ptr, u64_to_i64_ptr};

    #[test]
    fn test_i64_to_u64_zero() {
        let original: i64 = 0;
        let expected: u64 = 0;
        assert_eq!(i64_to_u64_ptr(original), expected);
    }

    #[test]
    fn test_u64_to_i64_positive() {
        let original: u64 = 0x7FFFFFFFFFFFFFFF; // Max positive i64 value
        let expected: i64 = 0x7FFFFFFFFFFFFFFF; // Same value as i64
        assert_eq!(u64_to_i64_ptr(original), expected);
    }

    #[test]
    fn test_u64_to_i64_negative() {
        let original: u64 = 0xFFFFFFFFFFFFFFFF; // Max u64 value, interpreted as -1 in i64
        let expected: i64 = -1;
        assert_eq!(u64_to_i64_ptr(original), expected);
    }

    #[test]
    fn test_i64_to_u64_positive() {
        let original: i64 = 0x7FFFFFFFFFFFFFFF; // Max positive i64 value
        let expected: u64 = 0x7FFFFFFFFFFFFFFF; // Same value as u64
        assert_eq!(i64_to_u64_ptr(original), expected);
    }

    #[test]
    fn test_i64_to_u64_negative() {
        let original: i64 = -1;
        let expected: u64 = 0xFFFFFFFFFFFFFFFF; // Max u64 value
        assert_eq!(i64_to_u64_ptr(original), expected);
    }
}
