//! Tests for the matchgen crate.
//!
//! This just includes `test-matchers.rs`, which is generated by `build.rs`. See
//! `tests/` and `benches/` for the actual test code.

#![forbid(unsafe_code)]

// Include generated code.
include!(concat!(env!("OUT_DIR"), "/test-matchers.rs"));
