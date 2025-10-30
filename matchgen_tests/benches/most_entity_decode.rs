//! Benchmark generated `most_entity_decode*` functions with [`criterion`].

#![allow(clippy::missing_docs_in_private_items, missing_docs)]

use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use matchgen_tests::{
    most_entity_decode_flat, most_entity_decode_iter, most_entity_decode_slice,
};
use std::time::Duration;

/// Helper for benchmarks.
macro_rules! benchmark {
    ( $group:expr, $test_name:expr, $input:expr ) => {{
        let input = $input;
        $group.throughput(Throughput::Bytes(input.len().try_into().unwrap()));
        $group.bench_with_input(
            BenchmarkId::new("most_entity_decode_iter", $test_name),
            input,
            |b, input| {
                b.iter(|| {
                    let mut iter = input.iter();
                    most_entity_decode_iter(&mut iter)
                })
            },
        );
        $group.bench_with_input(
            BenchmarkId::new("most_entity_decode_slice", $test_name),
            input,
            |b, input| b.iter(|| most_entity_decode_slice(input)),
        );
        $group.bench_with_input(
            BenchmarkId::new("most_entity_decode_flat", $test_name),
            input,
            |b, input| b.iter(|| most_entity_decode_flat(input)),
        );
    }};
}

fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("a");
    group
        .noise_threshold(0.10)
        .significance_level(0.01)
        .confidence_level(0.99)
        .sample_size(300)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(10));

    benchmark!(group, "timesbar", b"&timesbar;");
    benchmark!(
        group,
        "long_invalid",
        b"&xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
    );
    benchmark!(
        group,
        "long_none",
        b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;"
    );

    group.finish();
}

criterion_group!(unescape_group, benchmarks);
criterion_main!(unescape_group);
