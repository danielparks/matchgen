//! Test generated `match_nothing*_true` functions.

#![allow(clippy::missing_docs_in_private_items)]

use assert2::check;
use matchgen_tests::{match_nothing_slice_true, match_nothing_true};
use paste::paste;

macro_rules! test {
    ($name:ident, $input:expr, $result:expr, $remainder:expr) => {
        paste! {
            #[test]
            fn [<$name _iter>]() {
                let input = $input;
                let mut iter = input.iter();
                check!(match_nothing_true(&mut iter) == $result);
                check!(iter.as_slice() == $remainder);
            }

            #[test]
            fn [<$name _slice>]() {
                check!(
                    match_nothing_slice_true($input)
                    == ($result, $remainder.as_slice())
                );
            }
        }
    };
}

test!(nothing, b"", Some(true), b"");
test!(chars, b"abc", Some(true), b"abc");
