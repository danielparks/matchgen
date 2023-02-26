# Generate functions to quickly map byte string prefixes to values

[![docs.rs](https://img.shields.io/docsrs/iter-matcher)][docs.rs]
[![Crates.io](https://img.shields.io/crates/v/iter-matcher)][crates.io]
![Rust version 1.60+](https://img.shields.io/badge/Rust%20version-1.60%2B-success)

[`TreeMatcher`] can be used from a [build script] to generate a matcher
function. The function accepts an iterator over bytes and returns a mapped value
if it finds a given byte sequence at the start of the iterator.

For example, suppose you generate a [matcher for all HTML entities][htmlize]
called `entity_matcher()`:

```rust
let mut iter = b"&times;XYZ".iter();
assert!(entity_matcher(&mut iter) == Some("×"));
assert!(iter.next() == Some(b'X'));
```

  * **The prefixes it checks do not all have to be the same length.** The
    matcher will clone the iterator if it needs to look ahead, so when the
    matcher returns the iterator will only have consumed what was matched.
  * **If more than one prefix matches, it will return the longest one.**
  * **If nothing matches, the iterator will not be advanced.** You may want to
    call `iterator.next()` if the matcher returns `None`.
  * **It only checks the start of the iterator.** Often you will want to use
    [`position()`] or the [memchr crate][memchr] to find the start of a
    potential match.

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
        .render_iter(&mut out)?;

    Ok(())
}
```

To use the matcher:

```rust
include!(concat!(env!("OUT_DIR"), "/matcher.rs"));

fn main() {
    let mut iter = b"&amp; on &amp; on".iter();
    assert!(entity_decode(&mut iter) == Some(b'&'));
    assert!(iter.as_slice() == b" on &amp; on");
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
[`position()`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.position
[memchr]: http://docs.rs/memchr
[htmlize]: https://crates.io/crates/htmlize
