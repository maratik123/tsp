use std::num::IntErrorKind;
use std::ops::RangeBounds;
use std::str::FromStr;

pub fn is_alpha(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .all(|&c| matches!(c, b' '..=b'/' | b':'..=b'~'))
}

pub fn is_alphanum(bytes: &[u8]) -> bool {
    bytes.iter().all(|&c| matches!(c, b' '..=b'~'))
}

fn parse_internal(
    bytes: &[u8],
    allowed_len: impl RangeBounds<usize>,
    predicate: impl Fn(&[u8]) -> bool,
) -> Option<&str> {
    let bytes = trim_right_spaces(bytes);
    if allowed_len.contains(&bytes.len()) {
        fn sub_parse_internal(bytes: &[u8], predicate: impl Fn(&[u8]) -> bool) -> Option<&str> {
            if predicate(bytes) {
                std::str::from_utf8(bytes).ok()
            } else {
                None
            }
        }
        sub_parse_internal(bytes, predicate)
    } else {
        None
    }
}

pub fn parse_alpha(bytes: &[u8], allowed_len: impl RangeBounds<usize>) -> Option<&str> {
    parse_internal(bytes, allowed_len, is_alpha)
}

pub fn parse_alphanum(bytes: &[u8], allowed_len: impl RangeBounds<usize>) -> Option<&str> {
    parse_internal(bytes, allowed_len, is_alphanum)
}

pub fn trim_0d(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|&c| c != b'\r')
        .and_then(|left| {
            bytes
                .iter()
                .rposition(|&c| c != b'\r')
                .map(|right| &bytes[left..=right])
        })
        .unwrap_or_else(|| &bytes[..0])
}

// 5.1 All alpha and alpha/numeric fields will be left justified
pub fn trim_right_spaces(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .rposition(|&c| c != b' ')
        .map_or_else(|| &bytes[..0], |i| &bytes[..=i])
}

pub fn trim_leading_zeroes(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|&c| c != b'0')
        .map_or_else(|| &bytes[..0], |i| &bytes[i..])
}

macro_rules! parse_num_int_impl {
    ($($fn_name:ident $t:ty),+ $(,)?) => {$(
    pub fn $fn_name(bytes: &[u8], allowed_len: impl RangeBounds<usize>, allowed_range: impl RangeBounds<$t>) -> Option<$t> {
        fn parse_raw(bytes: &[u8]) -> Option<$t> {
            let bytes = trim_leading_zeroes(bytes);
            let s = std::str::from_utf8(bytes).ok()?;
            Some(match <$t>::from_str(s) {
                Ok(num) => num,
                Err(err) if err.kind() == &IntErrorKind::Empty => 0,
                _ => None?,
            })
        }
        fn parse_inner(bytes: &[u8], allowed_range: impl RangeBounds<$t>) -> Option<$t> {
            let raw_value = parse_raw(bytes)?;
            if allowed_range.contains(&raw_value) {
                Some(raw_value)
            } else {
                None?
            }
        }
        if allowed_len.contains(&bytes.len()) {
            parse_inner(bytes, allowed_range)
        } else {
            None
        }
    }
    )*}
}

parse_num_int_impl! {
    parse_num_u8 u8,
    parse_num_u16 u16,
    parse_num_u32 u32,
}

pub fn parse_blank(blank: u8) -> Option<()> {
    if blank == b' ' {
        Some(())
    } else {
        None
    }
}

pub fn parse_blank_arr(blank: &[u8], allowed_len: impl RangeBounds<usize>) -> Option<()> {
    if allowed_len.contains(&blank.len()) && blank.iter().all(|&c| c == b' ') {
        Some(())
    } else {
        None
    }
}
