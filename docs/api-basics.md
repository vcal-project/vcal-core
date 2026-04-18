---
id: api-basics
title: API Basics
sidebar_label: API Basics
---

The core types you’ll use most:

### `Index`
Create, configure, and query the vector index.

```rust
use vcal_core::{HnswBuilder, Cosine};

let mut idx = HnswBuilder::<Cosine>::default()
    .dims(768)
    .build()?;
```

### Key methods (common subset):

- **insert(InsertItem)** – insert if new id; error if exists
- **upsert(InsertItem)** – insert or replace by id
- **delete(&str)** – soft delete by id (tombstone)
- **search(&[f32], k)** – top-k nearest neighbors
- **batch_search(&[&[f32]], k)** – multiple queries in one call
- **size()** – approximate number of active vectors
- **contains(&str)** – check by id

### `InsertItem`
```rust
InsertItem::new("id", embedding)             // bare minimum
InsertItem::with_ttl("id", embedding, secs) // optional TTL per item
```

### Similarity metrics

- 'Cosine'
- 'Dot'

> Use Cosine for most embedding models; use Dot if your pipeline requires it.

### Errors

Most methods return:

`type VcalResult<T> = Result<T, VcalError>;`

Errors cover:
- invalid dimensions
- missing or duplicate IDs
- snapshot or internal index issues

> Note: Prior versions exposed an Index type. This has been replaced by Hnsw with a builder-based API.
