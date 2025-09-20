---
id: api-basics
title: API Basics
sidebar_label: API Basics
---

The core types you’ll use most:

### `Index`
Create, configure, and query the vector index.

```rust
let dims = 768;
let mut idx = Index::new(dims, Metric::Cosine)?;
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
``` rust
InsertItem::new("id", embedding)            // bare minimum
InsertItem::with_ttl("id", embedding, secs) // optional TTL on item
```

### Similarity `Metric`

- 'Metric::Cosine'
- 'Metric::Dot'

> Choose Cosine for typical embedding models; Dot if your pipeline expects it.

### Errors

Most methods return `VcalResult<T> = Result<T, VcalError>`.

Errors cover invalid dims, missing ids, or internal index issues.