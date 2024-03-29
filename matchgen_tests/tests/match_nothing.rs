//! Test generated `match_nothing*` functions.

#![allow(clippy::missing_docs_in_private_items)]

use assert2::check;
use matchgen_tests::{match_nothing, match_nothing_slice};
use paste::paste;

macro_rules! test {
    ($name:ident, $input:expr, $result:expr, $remainder:expr) => {
        paste! {
            #[test]
            fn [<$name _iter>]() {
                let input = $input;
                let mut iter = input.iter();
                check!(match_nothing(&mut iter) == $result);
                check!(iter.as_slice() == $remainder);
            }

            #[test]
            fn [<$name _slice>]() {
                check!(
                    match_nothing_slice($input)
                    == ($result, $remainder.as_slice())
                );
            }
        }
    };
}

test!(nothing, b"", None, b"");
test!(chars, b"abc", None, b"abc");
