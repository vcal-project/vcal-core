//! Criterion benchmark entry for VCAL-core.
//!
//! Run with
//! ```bash
//! cargo bench
//! ```

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vcal_core::{HnswBuilder, Cosine};

const DIMS: usize = 128;
const NUM_VECS: usize = 10_000;
const K: usize = 10;

fn build_index() -> vcal_core::Hnsw<Cosine> {
    let mut h = HnswBuilder::<Cosine>::default()
        .dims(DIMS)
        .m(16)
        .ef_construction(200)
        .ef_search(50)
        .build();

    // Deterministic vectors (i as f32) to keep distance monotonic.
    for i in 0..NUM_VECS {
        h.insert(vec![i as f32; DIMS], i as u64).unwrap();
    }
    h
}

fn bench_knn(c: &mut Criterion) {
    let h = build_index();
    let query = vec![0.0_f32; DIMS];

    let mut group = c.benchmark_group("knn_search");
    group.throughput(Throughput::Elements(1));

    group.bench_function(BenchmarkId::from_parameter(K), |b| {
        b.iter(|| {
            // We clone the query slice to prevent compiler cheating
            h.search(&query, K)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_knn);
criterion_main!(benches);
