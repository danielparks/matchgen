# Generate sequence matchers for iterators (experimental)

[![docs.rs](https://img.shields.io/docsrs/iter-matcher)][docs.rs]
[![Crates.io](https://img.shields.io/crates/v/iter-matcher)][crates.io]
![Rust version 1.60+](https://img.shields.io/badge/Rust%20version-1.60%2B-success)

This crate can be used from a [build script] to generate a matcher function. The
function accepts an iterator over bytes and returns a match if it finds a given
byte sequence at the start of the iterator.

The byte sequences it finds do not all have to be the same length. The matcher
will clone the iterator if it needs to look ahead, so when the matcher returns
the iterator will only have consumed what was matched.

Note that this means if nothing is matched the iterator will not move. You may
want to run `iterator.next()` if you’re running in a loop.

Note also that this does not search for the beginning of a match; it only checks
the start of the iterator. Often you will want to use [`position()`] or
the [memchr crate][memchr] to find the start of a potential match.

This is useful for things like [decoding HTML entities][htmlize]. To create a
matcher to handle the four basic HTML entities:

```rust
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let out_path = Path::new(&env::var("OUT_DIR")?).join("matcher.rs");
    let mut out = BufWriter::new(File::create(out_path)?);

    writeln!(out, "/// Decode basic HTML entities.")?;
    iter_matcher::Node::default()
        .add(b"&amp;", "b'&'")
        .add(b"&lt;", "b'<'")
        .add(b"&gt;", "b'>'")
        .add(b"&quot;", "b'\"'")
        .render(&mut out, "pub fn entity_decode", "u8")?;

    Ok(())
}
```

To use the matcher:

```rust
include!(concat!(env!("OUT_DIR"), "/test-matchers.rs"));

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
[build script]: https://doc.rust-lang.org/cargo/reference/build-scripts.html
[`position()`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.position
[memchr]: http://docs.rs/memchr
[htmlize]: https://crates.io/crates/htmlize
