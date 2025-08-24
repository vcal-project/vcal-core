//! knn_vs_redis.rs — Criterion A/B latency benchmark with console output.
//! Note: Redis comparison is a stub; enable feature `redis_bench` and add code if you need it.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use vcal_core::{HnswBuilder, Cosine};
use std::time::Instant;

const DIMS: usize = 128;
const NUM_VECS: usize = 10_000;
const K: usize = 10;

// ---------- VCAL helper --------------------------------------------------

fn build_vcal() -> vcal_core::Hnsw<Cosine> {
    let mut h = HnswBuilder::<Cosine>::default()
        .dims(DIMS)
        .build();

    for i in 0..NUM_VECS {
        h.insert(vec![i as f32; DIMS], i as u64).unwrap();
    }
    h
}

// ---------- Criterion bench ----------------------------------------------

fn bench_ab(c: &mut Criterion) {
    let mut group = c.benchmark_group("knn_vcal_vs_redis");
    let vcal = build_vcal();
    let query = vec![0.0_f32; DIMS];

    // Criterion measurement
    group.bench_with_input(BenchmarkId::new("vcal", K), &query, |b, q| {
        b.iter(|| vcal.search(q, K).unwrap())
    });
    group.finish();

    // Manual timing (console output)
    let runs = 10_000;
    let start = Instant::now();
    for _ in 0..runs {
        let _ = vcal.search(&query, K).unwrap();
    }
    let elapsed = start.elapsed();
    let per_query_ns = elapsed.as_nanos() as f64 / runs as f64;
    println!(
        "\n[Console] VCAL kNN search latency: {:.2} µs/query (k={})",
        per_query_ns / 1_000.0,
        K
    );

    #[cfg(feature = "redis_bench")]
    {
        println!("[Console] Redis benchmark output will be shown here if enabled.");
        // (Redis benchmarking code would go here if installed and feature enabled)
    }
}

criterion_group!(benches, bench_ab);
criterion_main!(benches);
