#![feature(test)]

extern crate test;

use iter_matcher::*;

#[bench]
fn rot13_basic(bench: &mut test::Bencher) {
    let source = "super secure";
    bench.iter(|| rot13(&source) );
    bench.bytes = source.len() as u64;
}
