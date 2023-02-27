# Generate functions to quickly map byte string prefixes to values

[![docs.rs](https://img.shields.io/docsrs/iter-matcher)][docs.rs]
[![Crates.io](https://img.shields.io/crates/v/iter-matcher)][crates.io]
![Rust version 1.60+](https://img.shields.io/badge/Rust%20version-1.60%2B-success)

[`TreeMatcher`] can be used from a [build script] to generate a matcher
function that maps byte sequences to arbitrary values. It returns the mapped
value (or `None`) and the remainder of the input.

For example, suppose you generate a [matcher for all HTML entities][htmlize]
called `entity_matcher()`:

```rust
assert!(entity_matcher(b"&times;XYZ") == (Some("×"), b"XYZ".as_slice()));
```

  * The prefixes it checks do not all have to be the same length.
  * If more than one prefix matches, it will return the longest one.
  * If nothing matches, it will return `(None, &input)`.

Since the matchers only check the start of the input, you will want to use
[`iter().position()`] or the [memchr crate][memchr] to find the start of a
potential match.

It can also be configured to accept an iterator over bytes as input instead of
a slice.

## Simple example

To create a matcher to handle the four basic HTML entities, use a build script
like the following:

```rust
use iter_matcher::TreeMatcher;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = Path::new(&env::var("OUT_DIR")?).join("matcher.rs");
    let mut out = BufWriter::new(File::create(out_path)?);

    writeln!(out, "/// Decode basic HTML entities.")?;
    TreeMatcher::new("pub fn entity_decode", "u8")
        .add(b"&amp;", "b'&'")
        .add(b"&lt;", "b'<'")
        .add(b"&gt;", "b'>'")
        .add(b"&quot;", "b'\"'")
        .render(&mut out)?;

    Ok(())
}
```

To use the matcher:

```rust
include!(concat!(env!("OUT_DIR"), "/matcher.rs"));

fn main() {
    assert_eq!(
      entity_decode(b"&amp; on &amp; on"),
      (Some(b'&'), b" on &amp; on".as_slice()),
    );
}
```

## License

This project dual-licensed under the Apache 2 and MIT licenses. You may choose
to use either.

  * [Apache License, Version 2.0](LICENSE-APACHE)
  * [MIT license](LICENSE-MIT)

### Contributions

Unless you explicitly state otherwise, any contribution you submit as defined
in the Apache 2.0 license shall be dual licensed as above, without any
additional terms or conditions.

[docs.rs]: https://docs.rs/iter-matcher/latest/iter_matcher/
[crates.io]: https://crates.io/crates/iter-matcher
[`TreeMatcher`]: https://docs.rs/iter-matcher/latest/iter_matcher/struct.TreeMatcher.html
[build script]: https://doc.rust-lang.org/cargo/reference/build-scripts.html
[`iter().position()`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.position
[memchr]: http://docs.rs/memchr
[htmlize]: https://crates.io/crates/htmlize
