---
id: changelog
title: Changelog (Library)
sidebar_label: Changelog
---

> This is a high-level changelog for **vcal-core**.  
> For full details, see Git history and GitHub Releases.

---

### v0.1.3 — April 2026

#### Security and Dependency Updates
- **Dependency hardening:** updated `rand` to `0.9.x` to address a RustSec advisory affecting earlier versions
- **Modernized randomness API:** migrated to the latest `rand` interfaces (`rng()` and `random()`)

**Notes**
- No changes to core algorithms or runtime behavior
- Safe upgrade recommended for all users due to dependency security improvements

---

### v0.1.2 — April 2026

#### API Safety and Validation
- **Builder validation:** `HnswBuilder::build()` now returns `Result` with explicit parameter checks
- **Stronger error handling:** new error types for invalid parameters and corrupted snapshots
- **Fail-fast validation:** invalid configurations are rejected early before index construction

#### Snapshot and Persistence Improvements
- **Safer serialization:** snapshot export now returns `Result<Vec<u8>>` (no panics)
- **Unified load path:** consistent validation and sanitization across all snapshot entry points
- **Improved robustness:** better handling of malformed or partially corrupted snapshot data

#### Architecture Changes
- **Explicit persistence model:** snapshot storage moved out of the core library
  - No built-in file handling (`save/load removed`)
  - Byte-based interface (`to_bytes` / `from_slice`) enables flexible integration
- **Fully safe Rust:** all unsafe code paths removed (`#![deny(unsafe_code)]`)

#### Algorithm Fixes and Stability
- **Corrected HNSW level generation:** fixed geometric distribution (`p = 1 / M`)
- **Improved graph stability:** prevents excessive upper-layer growth
- **Edge-case fixes:** resolved infinite loop risk when `M < 2`

**Notes**
- This release introduces breaking API changes in builder and persistence interfaces
- Designed to improve safety, predictability, and integration with higher-level systems like VCAL Server

---

### v0.1.1 — October 2025

#### Stability and Persistence Improvements
- **Envelope v1 Snapshot Format** introduced for safer index persistence  
  - Adds top-level `format`, `version`, `meta`, and `hnsw` fields  
  - Enables integrity validation and forward compatibility
- **Upsert Hardening:** automatic replacement of existing external IDs with correct graph rewiring  
- **Sanitize on Load:** legacy and mixed-format snapshots automatically repaired (dropped edges, fixed layers)
- **Safety Guards:** fixed potential out-of-bounds access when traversing deleted or missing neighbors
- **Improved Autosave Consistency:** paired atomic save (`index + answers`) now fully aligned with snapshot schema
- **Better Logging:** added `tracing::warn!` diagnostics for snapshot repair and I/O edge cases
- **No Performance Regression:** search latency and memory profile preserved vs. v0.1.0

#### Developer Notes
- Internal serialization moved to `serialize::to_bytes()` and `from_slice()` helpers  
- Compatible with `vcal-server >= 0.5.5` and all Envelope v1–based releases  
- Legacy (raw JSON) snapshots still load successfully, but re-save as Envelope v1

---

### v0.1.0 — September 2025

#### Initial public release

Improvements added to a pre-release version:
- HNSW improvements and memory tuning
- TTL and capacity-based eviction polished
- Snapshot save/load hardened
- Batch search performance bump
- Public API stabilization
- Error types consolidated
