---
id: changelog
title: Changelog (Library)
sidebar_label: Changelog
---

> This is a high-level changelog for **vcal-core**.  
> For full details, see Git history and GitHub Releases.

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
