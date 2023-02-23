use assert2::check;
use iter_matcher_tests::*;
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
