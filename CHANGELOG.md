# Change log

All notable changes to this project will be documented in this file.

## main branch

* Added `FlatMatcher`, which produces a single `match` statement on a slice
  rather than a tree of `match` statements like `TreeMatcher`.

  `FlatMatcher` can also be set to return the index of the next unmatched byte
  rather than a slice of the remainder, which automatically causes the resulting
  matcher to be `const`.
* Changed `TreeMatcher::new()` to accept `ToString` instead of
  `std::fmt::Display` since we use `to_string()`. `Display` already implied
  `ToString`, so this will not break compatibility.
* Made minor changes to decrease the MSRV from 1.60 to **1.56.1**.
* Fix a few spelling mistakes in documentation.

## Release 0.3.1 (2025-03-06)

* Added `TreeMatcher::write_to_path()` and `TreeMatcher::write_to_out_dir()` for
  convenience. This reduces boilerplate needed in build scripts.

## Release 0.3.0 (2024-06-18)

Once again, this release was cut to ensure generated code passes lints.

### Breaking changes

* Changed `TreeMatcher::new()` to accept `std::fmt::Display` instead of
  `AsRef<str>`, since we convert to a `String` with `to_string()` regardless.
  This is very unlikely to break anything.
* Changed `TreeNode::render_iter()` to accept `std::fmt::Display` instead of
  `AsRef<str>`, since we use the parameters in format strings. This is very
  unlikely to break anything.

### Changes

* Switched [code to disable Clippy][disable_clippy] to use `cfg(clippy)` instead
  of `cfg(feature = "cargo-clippy")`. The [feature has been deprecated].

[disable_clippy]: https://docs.rs/matchgen/latest/matchgen/struct.TreeMatcher.html#method.disable_clippy
[feature has been deprecated]: https://blog.rust-lang.org/2024/02/28/Clippy-deprecating-feature-cargo-clippy.html

## Release 0.2.0 (2023-05-26)

This release is primarily aimed at ensuring generated code passes lints.

### Breaking changes

* Renamed `TreeMatcher::set_input_type()` to `TreeMatcher::input_type()`.

### Changes

* Made sure generated functions pass [Clippy]’s [pedantic lint group]:
  * Changed to add [`#[must_use]`][must_use] to generated functions by default.
  * Added trivial documentation about what errors are returned by a few
    functions.
  * Disabled [`clippy::single_match_else`] lint within generated match
    functions, since we sometimes generate `match` statements with a single arm.
  * Disabled [`clippy::too_many_lines`] lint within generated match
    functions, since such functions can be extremely long.
* Added support for setting documentation on generated functions. This also adds
  docs to the stub function used in disabled Clippy mode which allows it to be
  used with [`#![warn(missing_docs)]`][missing_docs].

[Clippy]: https://doc.rust-lang.org/stable/clippy/index.html
[pedantic lint group]: https://doc.rust-lang.org/stable/clippy/usage.html#clippypedantic
[must_use]: https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute
[`clippy::single_match_else`]: https://rust-lang.github.io/rust-clippy/master/index.html#single_match_else
[`clippy::too_many_lines`]: https://rust-lang.github.io/rust-clippy/master/index.html#too_many_lines
[missing_docs]: https://doc.rust-lang.org/stable/nightly-rustc/rustc_lint/builtin/static.MISSING_DOCS.html

## Release 0.1.0 (2023-02-27)

### Features

* Initial release.
