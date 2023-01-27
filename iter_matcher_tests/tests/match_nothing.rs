use assert2::check;
use iter_matcher_tests::*;

macro_rules! test {
    ($name:ident, $input:expr, $result:expr, $remainder:expr) => {
        #[test]
        fn $name() {
            let input = $input;
            let mut iter = input.iter();
            check!(match_nothing_true(&mut iter) == $result);
            check!(iter.as_slice() == $remainder);
        }
    };
}

test!(nothing, b"", Some(true), b"");
test!(chars, b"abc", Some(true), b"abc");
