use assert2::check;
use iter_matcher_tests::*;

macro_rules! test {
    ($name:ident, $input:expr, $result:expr, $remainder:expr) => {
        #[test]
        fn $name() {
            let input = $input;
            let mut iter = input.iter();
            check!(basic_entity_decode(&mut iter) == $result);
            check!(iter.as_slice() == $remainder);
        }
    };
}

test!(nothing, b"", None, b"");
test!(chars, b"abc", None, b"abc");
test!(entity, b"&amp;", Some(b'&'), b"");
test!(entity_chars, b"&amp;abc", Some(b'&'), b"abc");
test!(entity_entity, b"&amp;&lt;", Some(b'&'), b"&lt;");
test!(short_entity_chars, b"&lt;abc", Some(b'<'), b"abc");
test!(invalid_entity, b"&invalid;", None, b"&invalid;");
