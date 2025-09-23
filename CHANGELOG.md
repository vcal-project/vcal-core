# Changelog
All notable changes to **vcal-core** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
