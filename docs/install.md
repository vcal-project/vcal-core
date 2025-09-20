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
vcal-core = "0.2"
serde = { version = "1", features = ["derive"] }
```
### Optional features

Enable extras via features:
```
[dependencies]
vcal-core = { version = "0.4", features = ["simd", "snapshots"] }
```
- simd – enable SIMD acceleration where available
- snapshots – enable snapshot (serde) helpers
