---
id: snapshots
title: Snapshots
sidebar_label: Snapshots
---

# Snapshots

Persist and restore a VCAL **HNSW** index using the optional `serde` feature.

> The snapshot format in **vcal-core** is an internal serialized structure of the `Hnsw` type.
> **VCAL Server** may wrap this in a *Snapshot Envelope v1* for durability and metadata.
> If you are embedding **vcal-core directly**, use the helpers below.

## Enable snapshot support

In your `Cargo.toml`:

```toml
[dependencies]
vcal-core = { version = "0.1.3", features = ["snapshots"] }
```

## Save to disk

```rust
use std::fs;
use vcal_core::{HnswBuilder, Cosine, InsertItem};

fn save_example() -> anyhow::Result<()> {
    let mut h = HnswBuilder::<Cosine>::default()
        .dims(16)
        .ef_search(64)
        .ef_construction(200)
        .build()?;

    h.insert(InsertItem::new("item:42", vec![1.0; 16]))?;

    // Serialize to bytes
    let bytes = h.to_bytes()?;
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

    // Restores the index with validation and sanitization
    let h = Hnsw::<Cosine>::from_slice(&bytes)?;

    Ok(h)
}
```

## Notes & recommendations

- **Atomic writes**: write to a temp file and rename to avoid partial snapshots on crash
- **Sanitization**: `from_slice` performs validation and repair (drops invalid edges, fixes layers)
- **Format stability**: the format may evolve between versions — treat it as **opaque**
  - For long-term durability, prefer **VCAL Server** snapshot endpoints (versioned envelope)
- **I/O cost**: saving large graphs can take seconds — keep snapshots off the hot path
- **Validation**: consider checksums or round-trip tests in CI

## Minimal round‑trip test

```rust
use vcal_core::{HnswBuilder, Hnsw, Cosine, InsertItem};

fn roundtrip() -> anyhow::Result<()> {
    let mut h = HnswBuilder::<Cosine>::default()
        .dims(8)
        .build()?;

    h.insert(InsertItem::new("item:7", vec![0.5; 8]))?;

    let bytes = h.to_bytes()?;
    let h2 = Hnsw::<Cosine>::from_slice(&bytes)?;

    let results = h2.search(&[0.5; 8], 1)?;
    assert_eq!(results.first().unwrap().id, "item:7");

    Ok(())
}
```
