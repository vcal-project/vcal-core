---
id: install
title: Install
sidebar_label: Install
---

## Prereqs
- Rust toolchain (stable)  
- `cargo` (bundled with Rustup)

## Add dependency
In your `Cargo.toml`:

```toml
[dependencies]
vcal-core = "0.1.3"
serde = { version = "1", features = ["derive"] }
```
### Optional features

Enable optional functionality via Cargo features:
```
[dependencies]
vcal-core = { version = "0.1.3", features = ["snapshots"] }
```
- `snapshots` — enables snapshot serialization via `serde` (`to_bytes` / `from_slice`)

**Notes**
- The library is **fully safe Rust** (`#![deny(unsafe_code)]``) and does not require SIMD or platform-specific features
- Snapshot support is **byte-based only** — you are responsible for storing and loading snapshot data (e.g., filesystem, object storage, database)