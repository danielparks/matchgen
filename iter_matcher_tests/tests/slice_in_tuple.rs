use assert2::check;
use iter_matcher_tests::*;

macro_rules! test {
    ($name:ident, $input:expr, $result:expr, $remainder:expr) => {
        #[test]
        fn $name() {
            let input = $input;
            let mut iter = input.iter();
            check!(slice_in_tuple(&mut iter) == $result);
            check!(iter.as_slice() == $remainder);
        }
    };
}

test!(nothing, b"", None, b"");
test!(invalid, b"ccc", None, b"ccc");
test!(aabc, b"aabc", Some((true, &[1, 1][..])), b"c");
test!(aac, b"aac", Some((false, &[1, 1][..])), b"c");
test!(abc, b"abc", Some((true, &[1][..])), b"c");
test!(ac, b"ac", Some((false, &[1][..])), b"c");
test!(aab, b"aab", Some((true, &[1, 1][..])), b"");
test!(aa, b"aa", Some((false, &[1, 1][..])), b"");
test!(ab, b"ab", Some((true, &[1][..])), b"");
test!(a, b"a", Some((false, &[1][..])), b"");
