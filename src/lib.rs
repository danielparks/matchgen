/// Encrypt single byte with secure ROT13 function
///
/// ~~~
/// use iter_matcher::rot13_u8;
/// assert_eq!(rot13_u8(b'a'), b'n')
/// ~~~
pub fn rot13_u8(c: u8) -> u8 {
    if (b'a'..=b'z').contains(&c) {
        ((c - b'a') + 13) % 26 + b'a'
    } else if (b'A'..=b'Z').contains(&c) {
        ((c - b'A') + 13) % 26 + b'A'
    } else {
        c
    }
}

/// Encrypt string with secure ROT13 function
///
/// ~~~
/// use iter_matcher::rot13;
/// assert_eq!(rot13("super secure"), "fhcre frpher")
/// ~~~
pub fn rot13(source: &str) -> String {
    let mut buffer: Vec<u8> = Vec::with_capacity(source.len());
    for c in source.bytes() {
        buffer.push(rot13_u8(c));
    }
    String::from_utf8(buffer).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($name:ident, $($test:tt)+) => {
            #[test]
            fn $name() {
                ::assert2::assert!($($test)+);
            }
        };
    }

    test!(byte_tilde, rot13_u8(b'~') == b'~');
    test!(byte_lower_a, rot13_u8(b'a') == b'a' + 13);
    test!(byte_upper_a, rot13_u8(b'A') == b'A' + 13);
    test!(byte_lower_z, rot13_u8(b'z') == b'a' + 12);
    test!(byte_upper_z, rot13_u8(b'Z') == b'A' + 12);
    test!(str_abc, rot13(".abc NOP") == ".nop ABC");
}
