---
id: quickstart
title: Quickstart
sidebar_label: Quickstart
---

A minimal example that inserts vectors and searches by cosine similarity.

```rust
use vcal_core::{Index, Metric, InsertItem};

fn main() -> anyhow::Result<()> {
    // 1) Create index
    let dims = 768;
    let mut idx = Index::new(dims, Metric::Cosine)?;

    // 2) Insert a few vectors (id + embedding)
    idx.insert(InsertItem::new("faq:001", vec![0.1; dims]))?;
    idx.insert(InsertItem::new("faq:002", vec![0.2; dims]))?;
    idx.insert(InsertItem::new("faq:003", vec![0.3; dims]))?;

    // 3) Search
    let query = vec![0.12; dims];
    let k = 2;
    let results = idx.search(&query, k)?;

    for hit in results {
        println!("id={} score={:.4}", hit.id, hit.score);
    }

    Ok(())
}
