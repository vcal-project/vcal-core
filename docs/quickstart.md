---
id: quickstart
title: Quickstart
sidebar_label: Quickstart
---

A minimal example that inserts vectors and searches by cosine similarity.

```rust
use vcal_core::{HnswBuilder, Cosine, InsertItem};

fn main() -> anyhow::Result<()> {
    // 1) Create index (Cosine similarity)
    let dims = 768;
    let mut idx = HnswBuilder::<Cosine>::default()
        .dims(dims)
        .build()?;

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
```

---

## What’s happening

### 1. Build the index
You create an HNSW graph configured for your embedding size and similarity metric.

- `dims` must match your embedding model (e.g. 384, 768, 1536)
- `Cosine` is the default choice for most LLM embeddings

---

### 2. Insert vectors
Each item has:
- a **string ID**
- an **embedding vector**

```rust
InsertItem::new("faq:001", embedding)
```

You can also:
- `upsert(...)` → replace existing items
- `delete(id)` → soft delete (tombstone)

---

### 3. Search
You query with a vector and get the **top-k nearest neighbors**:

```rust
let results = idx.search(&query, k)?;
```

Each result contains:
- `id` → your original identifier
- `score` → similarity score (higher = closer)

---

## How HNSW search works (intuition)

HNSW Search works like this:
1. Start from an entry point at the top layer  
2. Greedily move toward closer nodes  
3. Drop to lower layers for refinement  
4. Return the best `k` matches  

Result: fast approximate nearest neighbor search with high accuracy.

---

## Expected output

```text
id=faq:001 score=0.999x
id=faq:002 score=0.99xx
```

(Scores depend on your embeddings)

---

## Common next steps

- Add **TTL** to control memory usage  
- Use **batch_search()** for higher throughput  
- Persist the index via **snapshots (`to_bytes` / `from_slice`)**  
- Wrap in a service or use VCAL Server for production  
