use iai::black_box;
use iter_matcher_tests::*;

macro_rules! iai_benchmarks {
    ( $( ($name:ident, $input:expr), )+ ) => {
        $(
            fn $name() -> Option<&'static str> {
                let input = black_box($input);
                let mut iter = input.iter();
                black_box(all_entity_decode(&mut iter))
            }
        )+

        iai::main!(
            $($name,)+
        );
    }
}

iai_benchmarks! {
    (timesbar, b"&timesbar;"),
    (long_invalid, b"&xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
    (long_none, b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"),
}
