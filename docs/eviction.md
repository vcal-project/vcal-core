---
id: eviction
title: Eviction & TTL

sidebar_label: Eviction
---

## vcal-core supports **TTL** and **capacity-based** eviction

### TTL per item
Attach an expiration when inserting/upserting:

```rust
use vcal_core::InsertItem;
let item = InsertItem::with_ttl("faq:42", vec![...], 3600); // 1 hour
idx.upsert(item)?;
```
Expired items are lazily removed during mutations/queries.

### Capacity-based eviction (LRU-like)

Configure a max capacity and evict the least recently used items when full.
```
let mut idx = Index::with_capacity(768, Metric::Cosine, 200_000)?;
```
### Tips

- Start with no capacity, measure memory, then set a ceiling.
- Pair capacity with TTL for hot-set freshness.
