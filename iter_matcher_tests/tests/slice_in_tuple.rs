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
                check!(slice_in_tuple(&mut iter) == $result);
                check!(iter.as_slice() == $remainder);
            }

            #[test]
            fn [<$name _slice>]() {
                check!(
                    slice_in_tuple_slice($input)
                    == ($result, $remainder.as_slice())
                );
            }
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
