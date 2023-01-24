use criterion::{
    criterion_group, criterion_main, BenchmarkId, Criterion, Throughput,
};
use iter_matcher_tests::*;
use std::convert::TryInto;
use std::time::Duration;

// Helper for benchmarks.
macro_rules! benchmark {
    ( $group:expr, $test_name:expr, $input:expr ) => {{
        let input = $input;
        $group.throughput(Throughput::Bytes(input.len().try_into().unwrap()));
        $group.bench_with_input(
            BenchmarkId::new("all_entity_decode", $test_name),
            input,
            |b, input| b.iter(|| {
                let mut iter = input.iter();
                all_entity_decode(&mut iter)
            })
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
    benchmark!(group, "long_invalid", b"&xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
    benchmark!(group, "long_none", b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx;");

    group.finish();
}

criterion_group!(unescape_group, benchmarks);
criterion_main!(unescape_group);
