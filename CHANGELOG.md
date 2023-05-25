# Change log

All notable changes to this project will be documented in this file.

## main branch

* Added trivial documentation about what errors are returned by a few functions.
* Disabled [`clippy::single_match_else`] lint (part of [Clippy]’s [pedantic lint
  group]) within generated match functions, since we sometimes generate `match`
  statements with a single arm.

[`clippy::single_match_else`]: https://rust-lang.github.io/rust-clippy/master/index.html#single_match_else
[Clippy]: https://doc.rust-lang.org/stable/clippy/index.html
[pedantic lint group]: https://doc.rust-lang.org/stable/clippy/usage.html#clippypedantic

## Release 0.1.0 (2023-02-27)

### Features

* Initial release.
