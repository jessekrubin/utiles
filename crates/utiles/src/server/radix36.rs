//! Radix36 utils for server `request_id` mimics fastify's req-id

/// Convert u8 to radix36 char
/// ```
/// pub(crate)use utiles::server::radix36::u8_radix36_char;
/// assert_eq!(u8_radix36_char(0), '0');
/// assert_eq!(u8_radix36_char(9), '9');
/// assert_eq!(u8_radix36_char(10), 'a');
/// assert_eq!(u8_radix36_char(35), 'z');
/// ```
#[must_use]
#[inline]
pub fn u8_radix36_char(num: u8) -> char {
    if num < 10 {
        (b'0' + num) as char
    } else {
        (b'a' + num - 10) as char
    }
}

/// Radix36 for `request_id` mimics fastify's req-id
///
/// ```
/// use utiles::server::radix36::u64_radix36;
/// assert_eq!(u64_radix36(0), "0");
/// assert_eq!(u64_radix36(1234), "ya");
/// assert_eq!(u64_radix36(1109), "ut");
/// assert_eq!(u64_radix36(18446744073709551615), "3w5e11264sgsf");
/// ```
#[must_use]
pub fn u64_radix36(x: u64) -> String {
    if x < 36 {
        return u8_radix36_char(x as u8).to_string();
    }
    let mut result = ['\0'; 128];
    let mut used = 1;
    let mut x = x;
    let mut m = (x % 36) as u8;
    x /= 36;
    result[0] = u8_radix36_char(m);
    if x > 0 {
        loop {
            m = (x % 36) as u8;
            x /= 36;
            result[used] = u8_radix36_char(m);
            used += 1;
            if x == 0 {
                break;
            }
        }
    }
    let mut s = String::new();
    for c in result[..used].iter().rev() {
        s.push(*c);
    }
    s
}
