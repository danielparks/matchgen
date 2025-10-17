//! Benchmark building matchers with [`criterion`].
//!
//! This is mostly to ensure that it takes a reasonable amount of time.
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::too_many_lines,
    missing_docs
)]

use criterion::{criterion_group, criterion_main, Criterion};
use matchgen::{FlatMatcher, Input, TreeMatcher};
use std::fs;
use std::time::Duration;

fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder");
    group
        .noise_threshold(0.10)
        .significance_level(0.01)
        .confidence_level(0.99)
        .sample_size(300)
        .warm_up_time(Duration::from_millis(100))
        .measurement_time(Duration::from_secs(5));

    // Get HTML entities JSON
    let input = fs::read("matchgen_tests/most-html-entities.json").unwrap();
    let input: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(&input).unwrap();

    group.bench_with_input("iter", &input, |b, input| {
        b.iter(|| {
            let mut out = Vec::with_capacity(1_000_000);
            let mut matcher = TreeMatcher::new(
                "pub fn most_entity_decode_iter",
                "&'static str",
            );
            matcher
                .doc("Decode most HTML entities.\n\nIterator version.")
                .disable_clippy(true)
                .input_type(Input::Iterator)
                .extend(input.iter().map(|(name, info)| {
                    (
                        name.as_bytes(),
                        format!("{:?}", info["characters"].as_str().unwrap()),
                    )
                }));
            matcher.render(&mut out).unwrap();
            out
        });
    });

    group.bench_with_input("slice", &input, |b, input| {
        b.iter(|| {
            let mut out = Vec::with_capacity(1_000_000);
            let mut matcher = TreeMatcher::new(
                "pub fn most_entity_decode_slice",
                "&'static str",
            );
            matcher
                .doc("Decode most HTML entities.\n\nSlice version.")
                .collapse_nested_single_arms(false)
                .disable_clippy(true)
                .input_type(Input::Slice)
                .extend(input.iter().map(|(name, info)| {
                    (
                        name.as_bytes(),
                        format!("{:?}", info["characters"].as_str().unwrap()),
                    )
                }));
            matcher.render(&mut out).unwrap();
            out
        });
    });

    group.bench_with_input("slice_collapse", &input, |b, input| {
        b.iter(|| {
            let mut out = Vec::with_capacity(1_000_000);
            let mut matcher = TreeMatcher::new(
                "pub fn most_entity_decode_slice_collapse",
                "&'static str",
            );
            matcher
                .doc("Decode most HTML entities.\n\nSlice collapse version.")
                .collapse_nested_single_arms(true)
                .disable_clippy(true)
                .input_type(Input::Slice)
                .extend(input.iter().map(|(name, info)| {
                    (
                        name.as_bytes(),
                        format!("{:?}", info["characters"].as_str().unwrap()),
                    )
                }));
            matcher.render(&mut out).unwrap();
            out
        });
    });

    group.bench_with_input("flat", &input, |b, input| {
        b.iter(|| {
            let mut out = Vec::with_capacity(1_000_000);
            let mut matcher = FlatMatcher::new(
                "pub fn most_entity_decode_flat",
                "&'static str",
            );
            matcher
                .doc("Decode most HTML entities.\n\nFlat match slice version.")
                .disable_clippy(true)
                .extend(input.iter().map(|(name, info)| {
                    (
                        name.as_bytes(),
                        format!("{:?}", info["characters"].as_str().unwrap()),
                    )
                }));
            matcher.render(&mut out).unwrap();
            out
        });
    });

    group.bench_with_input("flat_const", &input, |b, input| {
        b.iter(|| {
            let mut out = Vec::with_capacity(1_000_000);
            let mut matcher = FlatMatcher::new(
                "pub fn most_entity_decode_flat_const",
                "&'static str",
            );
            matcher
                .doc(
                    "Decode most HTML entities.\n\
                    \n\
                    Const flat match slice version.",
                )
                .disable_clippy(true)
                .return_index()
                .extend(input.iter().map(|(name, info)| {
                    (
                        name.as_bytes(),
                        format!("{:?}", info["characters"].as_str().unwrap(),),
                    )
                }));
            matcher.render(&mut out).unwrap();
            out
        });
    });

    group.finish();
}

criterion_group!(unescape_group, benchmarks);
criterion_main!(unescape_group);
