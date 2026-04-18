---
id: eviction
title: Eviction & TTL

sidebar_label: Eviction
---

vcal-core supports TTL and capacity-based eviction to control memory usage.

### TTL per item

Attach an expiration when inserting/upserting:

```rust
use vcal_core::InsertItem;

let item = InsertItem::with_ttl("faq:42", vec![...], 3600); // 1 hour
idx.upsert(item)?;
```
- Expired items are skipped during search
- Removal is lazy and happens during queries or mutations

### Capacity-based eviction

Limit the number of active vectors in the index.

```rust
use vcal_core::{HnswBuilder, Cosine};

let mut idx = HnswBuilder::<Cosine>::default()
    .dims(768)
    .max_capacity(200_000) // example name
    .build()?;
```

When capacity is reached:

- older / less recently used items are evicted
- behavior is approximate (recency-aware), not strict LRU

### Eviction model

vcal-core does not run background workers.

Eviction happens:
- during inserts / upserts
- during search operations

This keeps the library:
- predictable
- lightweight
- easy to embed

### Tips

- Start without a capacity limit, measure memory usage, then set a ceiling
- Combine TTL + capacity for better control of hot vs stale data
- Tune eviction behavior in your application if you need strict policies