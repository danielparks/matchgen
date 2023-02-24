use iai::black_box;
use iter_matcher_tests::*;
use paste::paste;

macro_rules! iai_benchmarks {
    ( $( ($name:ident, $input:expr), )+ ) => {
        paste! {
            $(
                fn [<$name _iter>]() -> Option<&'static str> {
                    let input = black_box($input);
                    let mut iter = input.iter();
                    black_box(all_entity_decode(&mut iter))
                }

                fn [<$name _slice>]() -> (Option<&'static str>, &'static [u8]) {
                    black_box(all_entity_decode_slice(
                        black_box($input.as_slice())
                    ))
                }
            )+

            iai::main!(
                $([<$name _iter>], [<$name _slice>],)+
            );
        }
    }
}

iai_benchmarks! {
    (timesbar, b"&timesbar;"),
    (long_invalid, b"&xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
    (long_none, b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
}
