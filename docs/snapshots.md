---
id: snapshots
title: Snapshots
sidebar_label: Snapshots
---

## Persist the index to disk and restore on startup

```rust
use vcal_core::{Index, Metric};
use std::fs::File;

fn save(idx: &Index, path: &str) -> anyhow::Result<()> {
    let f = File::create(path)?;
    idx.save(f)?; // requires "snapshots" feature
    Ok(())
}

fn load(path: &str) -> anyhow::Result<Index> {
    let f = File::open(path)?;
    let idx = Index::load(f)?; // requires "snapshots" feature
    Ok(idx)
}
```
### Notes
- Snapshot includes vectors, ids, tombstones, and metadata.
- For large indexes, snapshot I/O can take seconds—schedule accordingly.
- Store alongside your app data directory; rotate files if needed.