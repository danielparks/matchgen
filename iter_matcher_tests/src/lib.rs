#[cfg(not(feature = "cargo-clippy"))]
include!(concat!(env!("OUT_DIR"), "/test-matchers.rs"));

// Stubs to make clippy happy. It is extremely slow with the real functions. See
// https://github.com/rust-lang/rust-clippy/issues/10220

macro_rules! stub {
    ($name:ident, $ret:ty) => {
        #[cfg(feature = "cargo-clippy")]
        pub fn $name<'a, I>(iter: &mut I) -> Option<$ret>
        where
            I: Iterator<Item = &'a u8> + Clone,
        {
            // Just in case this actually gets used
            iter.next();
            None
        }
    };
}

stub!(all_entity_decode, &'static str);
stub!(basic_entity_decode, u8);
stub!(match_nothing, u8);
stub!(match_nothing_true, bool);
