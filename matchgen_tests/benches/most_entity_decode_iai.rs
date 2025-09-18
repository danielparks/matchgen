//! Benchmark generated `most_entity_decode*` functions with [`iai`].

#![allow(clippy::missing_docs_in_private_items)]

use iai::black_box;
use matchgen_tests::{
    most_entity_decode, most_entity_decode_flat, most_entity_decode_slice,
};
use paste::paste;

/// Helper for benchmarks.
macro_rules! iai_benchmarks {
    ( $( ($name:ident, $input:expr), )+ ) => {
        paste! {
            $(
                fn [<$name _iter>]() -> Option<&'static str> {
                    let input = black_box($input);
                    let mut iter = input.iter();
                    black_box(most_entity_decode(&mut iter))
                }

                fn [<$name _slice>]() -> (Option<&'static str>, &'static [u8]) {
                    black_box(most_entity_decode_slice(
                        black_box($input.as_slice())
                    ))
                }

                fn [<$name _flat>]() -> (Option<&'static str>, &'static [u8]) {
                    black_box(most_entity_decode_flat(
                        black_box($input.as_slice())
                    ))
                }
            )+

            iai::main!(
                $([<$name _iter>], [<$name _slice>], [<$name _flat>],)+
            );
        }
    }
}

iai_benchmarks! {
    (timesbar, b"&timesbar;"),
    (long_invalid, b"&xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
    (long_none, b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
}
