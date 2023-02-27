use assert2::check;
use matchgen_tests::*;
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
