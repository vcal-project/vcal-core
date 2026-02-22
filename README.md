<p align="center">
  <a href="https://vcal-project.com" target="_blank" rel="noopener">Website</a> ·
  <a href="https://vcal-project.com/#pricing" target="_blank" rel="noopener">Pricing</a> ·
  <a href="https://vcal-project.com/#contact" target="_blank" rel="noopener">Pilot Access</a>
</p>

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](#)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)](#)

# ![VCAL mark](docs/assets/vcal-favicon.png) VCAL-core

**VCAL-core** is a lightweight, in-process [HNSW](https://arxiv.org/abs/1603.09320) vector index written in safe Rust with optional SIMD and atomic snapshots.  
It’s designed as a minimal building block for **semantic caches** (e.g., LLM prompt deduplication) and embedded ANN search.

> **MSRV:** 1.56 (edition 2021). *Dev-dependencies may require a newer stable toolchain.*

---

## Why VCAL-core?

- **Ultra-light:** minimal dependencies, no `unsafe` in public API.  
- **Fast enough:** competitive k-NN for small/mid indexes (edge or cache use).  
- **Embeddable:** no daemon — runs directly inside your process.  
- **Deterministic:** single-threaded core; wrap in `RwLock` for concurrency.  
- **Persistent:** safe paired snapshots and simple TTL/LRU eviction.  

---

## Key Features

- **HNSW index** with greedy descent + `ef_search`  
- **Pluggable metrics:** `Cosine` (default), `Dot`  
- **Optional SIMD** (`--features simd` + `RUSTFLAGS="-C target-cpu=native"`)  
- **Snapshots:** binary persistence with the `serde` feature  
  - Atomic paired saves prevent corruption (`.index.A` / `.index.B`)  
  - Automatic recovery from latest intact snapshot  
- **Eviction:**  
  - `evict_ttl(ttl_secs)` — remove expired entries  
  - `evict_lru_until(max_vectors, max_bytes)` — respect soft caps  
- **Stats:** `stats()` → `(vector_count, approx_bytes)`  
- **Simple API:** `insert`, `delete`, `contains`, and `search`  

---

## Install

```toml
[dependencies]
vcal-core = { version = "0.1.1", features = ["serde"] }
```

Optional features:
- `serde` — enable snapshot persistence  
- `simd` — AVX2-optimized inner loops (`x86_64` only)  

---

## Quick Example

```rust
use vcal_core::{HnswBuilder, Cosine};

let mut idx = HnswBuilder::<Cosine>::default().dims(128).build();
idx.insert(vec![0.1; 128], 1001)?;
let hits = idx.search(&vec![0.1; 128], 5)?;
```

---

## Persistence (v0.1.1)

```rust
use vcal_core::Index;
use std::fs::File;

let idx = Index::new(...)?;
let f = File::create("vcal.index")?;
idx.save(f)?; // alternates between paired files safely
```

Paired saves guarantee atomic recovery: on restart, `load()` automatically picks the last valid `.index` version.

---

## Eviction

```rust
idx.evict_ttl(3600);                        // Remove expired entries
idx.evict_lru_until(Some(1000), None);      // Keep up to 1000 vectors
```

---

## Observability

`vcal-core` itself is metrics-agnostic.  
For Prometheus and Grafana integration, use **vcal-server**, which exposes `/metrics`.

---

## Performance Tips

- Build in release mode:  
  `cargo build --release`
- Use native SIMD:  
  `RUSTFLAGS="-C target-cpu=native" cargo build --release --features simd`
- Normalize embeddings for cosine metric.
- Typical parameters:  
  `m = 16–32`, `ef_search = 64–256`.

---

## Design Principles

- No background threads or implicit I/O.  
- No public `unsafe`.  
- Snapshot format is stable for 0.x line, versioned from `v0.1.0`.  
- Optimized for **embedded** and **server-local** caches, not massive-scale ANN.  

---

For Python and chatbot integration, see [INSTALL.md](INSTALL.md).

---

## License

Licensed under **Apache-2.0**. See `LICENSE-Apache-2.0`  
© 2026 VCAL-project contributors
