---
id: snapshots
title: Snapshots
sidebar_label: Snapshots
---

# Snapshots

Persist and restore a VCAL **HNSW** index using the optional `serde` feature.

> The raw snapshot format in **vcal-core** is an internal JSON structure of the `Hnsw` type.  
> The **VCAL Server** may wrap this in a *Snapshot Envelope v1* for durability & metadata.  
> If you’re embedding **vcal-core** directly, use the helpers below.

## Enable the `serde` feature

In your `Cargo.toml`:

```toml
[dependencies]
vcal-core = { version = "0.1", features = ["serde"] }
```

## Save to disk

```rust
use std::fs;
use vcal_core::{HnswBuilder, Cosine};

fn save_example() -> anyhow::Result<()> {
    let mut h = HnswBuilder::<Cosine>::default()
        .dims(16)
        .ef_search(64)
        .ef_construction(200)
        .build();

    h.insert(vec![1.0; 16], 42)?;

    // Serialize to bytes (pretty JSON by default)
    let bytes = h.to_bytes();
    fs::write("hnsw.snapshot", &bytes)?;
    Ok(())
}
```

## Load from disk

```rust
use std::fs;
use vcal_core::{Hnsw, Cosine};

fn load_example() -> anyhow::Result<Hnsw<Cosine>> {
    let bytes = fs::read("hnsw.snapshot")?;
    // Restores the index and performs a light sanitize pass
    let h = Hnsw::<Cosine>::from_slice(&bytes)?;
    Ok(h)
}
```

## Notes & recommendations

- **Atomic writes**: write to a temp file and rename to avoid partial snapshots on crash.
- **Sanitization**: `from_slice` performs a light repair pass (drops bad edges, fixes empty levels).
- **Format stability**: the library format may evolve between minor versions; treat it as *opaque*.
  - For durable, operational snapshots, prefer the **VCAL Server** endpoints which emit a versioned envelope.
- **I/O cost**: saving large graphs can take seconds—schedule snapshots out of the hot path.
- **Validation**: consider checksumming the file or verifying it round-trips in CI for regression coverage.

## Minimal round‑trip test

```rust
use vcal_core::{HnswBuilder, Hnsw, Cosine};

fn roundtrip() -> anyhow::Result<()> {
    let mut h = HnswBuilder::<Cosine>::default().dims(8).build();
    h.insert(vec![0.5; 8], 7)?;
    let bytes = h.to_bytes();
    let h2 = Hnsw::<Cosine>::from_slice(&bytes)?;
    assert_eq!(h2.search(&[0.5; 8], 1)?.first().unwrap().0, 7);
    Ok(())
}
```
