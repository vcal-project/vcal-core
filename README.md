<p align="center">
  <a href="https://vcal-project.com" target="_blank" rel="noopener">Website</a> ·
  <a href="https://vcal-project.com/vcal-server/#pricing" target="_blank" rel="noopener">Pricing</a> ·
  <a href="https://vcal-project.com/vcal-server/#contact" target="_blank" rel="noopener">Pilot Access</a> ·
  <a href="https://docs.vcal-project.com/" target="_blank" rel="noopener">Docs</a>
</p>

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](#)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)](#)
[![MSRV](https://img.shields.io/badge/MSRV-1.56-blue)](#)
[![LLM Infra](https://img.shields.io/badge/LLM-infrastructure-black)](#)

# ![VCAL mark](docs/assets/vcal-favicon.png) VCAL-core

**VCAL-core** is a lightweight, in-process [HNSW](https://arxiv.org/abs/1603.09320) vector index wwritten in safe Rust with optional snapshot support.  

It’s designed as a minimal building block for **semantic caches** (e.g., LLM prompt deduplication) and embedded ANN search.

> **MSRV:** 1.56 (edition 2021). *Dev-dependencies may require a newer stable toolchain.*

---

## Why VCAL-core?

- **Ultra-light:** minimal dependencies, fully safe Rust (`#![deny(unsafe_code)]`)
- **Embeddable:** no daemon, runs directly inside your process
- **Predictable:** no background threads or hidden I/O
- **Practical performance:** optimized for small/mid-sized indexes (edge / cache use)
- **Production-friendly:** explicit errors, deterministic behavior

---

## Key Features


- **HNSW index**
  - greedy descent + configurable `ef_search`
  - corrected geometric level generation (v0.1.2)

- **Pluggable metrics**
  - `Cosine` (default)
  - `Dot`

- **Safe snapshot support** *(optional via `serde`)*
  - JSON-based persistence
  - no panic on serialization
  - validated + sanitized on load

- **Eviction**
  - `evict_ttl(ttl_secs)` — remove expired entries  
  - `evict_lru_until(max_vectors, max_bytes)` — respect soft caps  

- **Stats**
  - `stats()` → `(vector_count, approx_bytes)`

- **Simple API**
  - `insert`, `delete`, `contains`, `search`

 ---

## Install

```toml
[dependencies]
vcal-core = { version = "0.1.2", features = ["serde"] }
```

Optional features:
- `serde` — enable snapshot persistence

> `vcal-core` is a Rust library that is currently distributed via GitHub and not yet published on crates.io.

> SIMD support is intentionally deferred in v0.1.2 to keep the crate fully safe and minimal.

---

## Quick Example

```rust
use vcal_core::{HnswBuilder, Cosine};

let mut idx = HnswBuilder::<Cosine>::default()
    .dims(128)
    .build()
    .unwrap();

idx.insert(vec![0.1; 128], 1001).unwrap();

let hits = idx.search(&[0.1; 128], 5).unwrap();
assert_eq!(hits[0].0, 1001);
```

---

## Snapshots (optional)

```rust
use vcal_core::{HnswBuilder, Cosine};

let mut idx = HnswBuilder::<Cosine>::default()
    .dims(128)
    .build()
    .unwrap();

let bytes = idx.to_bytes().unwrap();
let restored = vcal_core::from_slice::<Cosine>(&bytes).unwrap();
```

Notes:
- snapshot uses `serde_json`
- loading performs internal validation and graph sanitization
- errors are returned (no panics)

---

## Eviction

```rust
idx.evict_ttl(3600);                        // remove expired entries
idx.evict_lru_until(Some(1000), None);      // keep up to 1000 vectors
```

---

## Observability

`vcal-core` is intentionally metrics-agnostic.

For:
- Prometheus
- Grafana dashboards
- cost tracking

— use [**VCAL Server**](https://vcal-project.com/vcal-server/) (higher-level component).

---

## Performance Tips

- Build in release mode:
  ```bash
  cargo build --release
  ```

- Normalize vectors when using cosine similarity

- Typical parameters:
  - `m = 16–32`
  - `ef_search = 64–256`

---

## Design Principles

- No public `unsafe`
- No hidden concurrency
- No implicit I/O
- Explicit error handling (`Result<T, VcalError>`)
- Optimized for **embedded semantic caching**, not massive-scale ANN

---

## Security Notes

- `cargo audit` may report `RUSTSEC-2026-0097` for `rand 0.8.x`
- current usage is limited to internal sampling (no known exposure path)
- dependency will be upgraded in a future release

---

## Roadmap

- SIMD fast path (safe abstraction)
- improved snapshot formats
- additional distance metrics
- better tuning diagnostics

---

For production deployment, observability, and persistence, see [**VCAL Server**](https://vcal-project.com/vcal-server/)

---

## License

Licensed under **Apache-2.0**. See `LICENSE-Apache-2.0`  
© 2026 VCAL-project contributors
