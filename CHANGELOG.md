# Changelog
All notable changes to **vcal-core** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-04-14

### Added
- **Builder validation hardening**
  - `HnswBuilder::build()` now returns `Result<Hnsw<_>>`
  - Explicit validation for required parameters (e.g., `dims > 0`)
  - New error variant: `InvalidDimensions { found }`

- **Improved error surface**
  - Added `InvalidParameter(&'static str)` for future parameter validation
  - Added `CorruptSnapshot(String)` (under `serde` feature)
  - Consistent error handling across builder, snapshot, and search paths

- **Snapshot safety improvements**
  - `to_bytes()` now returns `Result<Vec<u8>>` (no panics)
  - Unified snapshot load path with consistent validation and sanitization
  - Added tests for invalid JSON and malformed snapshot handling

---

### Changed
- **Removed unsafe feature paths**
  - SIMD support deferred to future release
  - Crate is now fully safe (`#![deny(unsafe_code)]`) across all features

- **Corrected HNSW level generation**
  - Fixed geometric distribution for layer assignment (`p = 1 / M`)
  - Prevents excessive upper-layer growth and improves search behavior

- **Snapshot model simplified**
  - Removed paired snapshot (`.A / .B`) mechanism
  - Persistence is now explicit byte-based (`to_bytes` / `from_slice`)
  - Storage and durability responsibility moved to higher-level systems (e.g., VCAL Server)

- **API consistency improvements**
  - Unified snapshot loading behavior across all entry points
  - Improved internal validation ordering (fail fast before mutation)

---

### Fixed
- Eliminated potential infinite loop when `M < 2` in level generation
- Fixed inconsistent snapshot load behavior between public APIs
- Resolved panic-prone serialization path (`expect()` removed)
- Cleaned up benchmark and test inconsistencies after API changes

---

### Removed
- Paired snapshot persistence (`Index::save`, `.index.A/.index.B`)
- File-based snapshot API (`save/load`)
- SIMD feature flag (`simd`)
- Legacy `Index` abstraction (fully migrated to `Hnsw`)

---

### Migration Notes
- **Builder now returns `Result`:**

```rust
let idx = HnswBuilder::<Cosine>::default()
    .dims(128)
    .build()
    .unwrap();
```

---

## [0.1.1] - 2025-10-10
### Added
- **Paired snapshot mechanism** to ensure atomic persistence:
  - `Index::save()` now safely alternates between paired `.index` files to prevent corruption on crash or power loss.
  - Snapshot loading automatically picks the latest intact version.
- **Integrated autosave hooks** (when enabled by server or wrapper):
  - Designed for background persistence during runtime or graceful shutdown.
- **New internal metrics counters** for TTL and LRU evictions (available to higher layers).
- **Expanded tests and benchmarks** for concurrent upsert, search, and snapshot recovery.

### Changed
- **Improved eviction precision**: TTL and LRU eviction now return consistent `(removed, freed_bytes)` across runs.
- **Optimized snapshot I/O** to minimize memory copies on large datasets.
- **Refined in-memory stats** — more accurate accounting of `approx_bytes` and reduced overcount on vector reuse.
- **Simplified API surface** for snapshot and persistence features — aligned with VCAL Server persistence layer.

### Fixed
- Resolved rare panic on startup when loading incomplete `.index` file.
- Fixed internal timestamp drift that could affect TTL expiration in long-running sessions.
- Eliminated data race conditions during simultaneous TTL + LRU sweeps.

### Migration Notes
- Existing `.index` files remain compatible; paired saves introduce no format changes.
- To enable paired persistence in your app or service:
  ```rust
  use vcal_core::Index;
  use std::fs::File;

  let idx = Index::new(...)?;
  let f = File::create("vcal.index")?;
  idx.save(f)?; // alternates between .index.A and .index.B internally
  ```
- Always stop background writers before shutdown to ensure both pairs stay consistent.

---

## [0.1.0] - 2025-09-02
### Added
- **Snapshot serialization** (requires `serde` feature):
  - `Hnsw::<Cosine>::to_bytes()` — serialize an index to a compact binary blob.
  - `Hnsw::<Cosine>::from_slice(&[u8])` — reconstruct an index from a blob.
  - Note: snapshot format may evolve until `1.0.0`.
- **Eviction primitives** for capacity management:
  - `evict_ttl(ttl_secs: u64) -> (usize, usize)` — remove entries older than TTL (returns `(removed, freed_bytes)`).
  - `evict_lru_until(max_vectors: Option<usize>, max_bytes: Option<usize>) -> (usize, usize)` — LRU-evict until soft caps are met.
- **ID management & introspection**:
  - `delete(ext_id: u64) -> bool` — idempotent removal; `true` if an item existed.
  - `contains(ext_id: u64) -> bool` — quick membership check.
  - `stats() -> (usize, usize)` — `(vector_count, approx_bytes)` for planning/observability.
- **Builder ergonomics**:
  - `HnswBuilder::<Cosine>::dims(usize).m(usize).build()`.

### Changed
- Clarified **upsert pattern**: prefer `delete(ext_id)` followed by `insert(vector, ext_id)` when replacing.
- Improved memory accounting used by `stats()` and LRU eviction.
- Consolidated error variants to map cleanly to API surfaces:
  - `EmptyIndex` and `DimensionMismatch { expected, found }` retained; other internals map to a generic internal error upstream.

### Fixed
- Precise diagnostics for **dimension mismatch**.
- Stable ordering for equal-distance results to reduce flaky tests.
- Safer handling of empty / near-empty indices in `search` and `stats`.

### Security
- Snapshot load is pure data (no code execution).
- Eviction APIs touch only in-memory state; no external I/O.

### Migration Notes
- Enable snapshotting explicitly:
  ```toml
  vcal-core = { version = "0.2", features = ["serde"] }
  ```
- For upsert semantics:
  ```rust
  idx.delete(ext_id);
  idx.insert(vector, ext_id)?;
  ```
- Treat snapshot blobs as versioned but **not yet stable** until `1.0.0`.

## [0.0.1] - 2025-08-01
### Added
- Initial public release with HNSW (Cosine): `HnswBuilder`, `Hnsw`, `insert`, `search`.
- Basic error types and dimension checks.

---

[Unreleased]: https://github.com/vcal-project/vcal-core/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/vcal-project/vcal-core/compare/v0.1.0...v0.2.0
