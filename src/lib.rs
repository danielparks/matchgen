//! These matcher builders can be used from a [build script] to generate
//! functions that accept bytes as an input and return a mapped value if they
//! find a given byte sequence at the start of the input.
//!
//! [`TreeMatcher`] generates more complicated but often faster code, while
//! [`FlatMatcher`] generates simpler but often slower code. See their
//! documentation for example usage.
//!
//! If you need a `const fn` matcher, then your only option is to use
//! [`FlatMatcher`] with [`FlatMatcher::return_index()`], which causes the
//! generated function to return the index of the next unmatched byte instead of
//! a slice. That allows the function to be `const`.
//!
//! # Minimum supported Rust version
//!
//! Currently the minimum supported Rust version (MSRV) is **1.56.1**.
//!
//! [build script]: https://doc.rust-lang.org/cargo/reference/build-scripts.html

// Lint configuration in Cargo.toml isn’t supported by cargo-geiger.
#![forbid(unsafe_code)]

mod flat;
mod tree;

pub use flat::*;
pub use tree::*;
