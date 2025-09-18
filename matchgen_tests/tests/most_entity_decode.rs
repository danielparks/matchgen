//! Test generated `most_entity_decode*` functions.

#![allow(clippy::missing_docs_in_private_items)]

use assert2::check;
use matchgen_tests::{
    most_entity_decode, most_entity_decode_flat, most_entity_decode_slice,
};
use paste::paste;

macro_rules! test {
    ($name:ident, $input:expr, $result:expr, $remainder:expr) => {
        paste! {
            #[test]
            fn [<$name _iter>]() {
                let input = $input;
                let mut iter = input.iter();
                check!(most_entity_decode(&mut iter) == $result);
                check!(iter.as_slice() == $remainder);
            }

            #[test]
            fn [<$name _slice>]() {
                check!(
                    most_entity_decode_slice($input)
                    == ($result, $remainder.as_slice())
                );
            }

            #[test]
            fn [<$name _flat>]() {
                check!(
                    most_entity_decode_flat($input)
                    == ($result, $remainder.as_slice())
                );
            }
        }
    };
}

test!(nothing, b"", None, b"");
test!(chars, b"abc", None, b"abc");
test!(entity, b"&amp;", Some("&"), b"");
test!(entity_chars, b"&amp;abc", Some("&"), b"abc");
test!(entity_entity, b"&amp;&lt;", Some("&"), b"&lt;");
test!(short_entity_chars, b"&lt;abc", Some("<"), b"abc");

test!(timesbar, b"&timesbar;", Some("⨱"), b"");
test!(timesb, b"&timesb;", Some("⊠"), b"");
test!(times, b"&times;", Some("×"), b"");
test!(times_bare_b, b"&timesb", Some("×"), b"b");
