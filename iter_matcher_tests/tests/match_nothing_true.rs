use assert2::check;
use iter_matcher_tests::*;

macro_rules! test {
    ($name:ident, $input:expr, $result:expr, $remainder:expr) => {
        #[test]
        fn $name() {
            let input = $input;
            let mut iter = input.iter();
            check!(match_nothing(&mut iter) == $result);
            check!(iter.as_slice() == $remainder);
        }
    };
}

test!(nothing, b"", None, b"");
test!(chars, b"abc", None, b"abc");
