---
id: faq
title: FAQ
sidebar_label: FAQ
---

**Is vcal-core a database?**  
No. It’s an **in-process** vector index library. Embed it directly into your service for fast, local semantic search.

**Cosine vs Dot?**  
Use **Cosine** for most text embeddings.
Use **Dot** if your embedding pipeline is designed for it.

**Can I delete items?**  
Yes: `delete(id)` marks the item as a tombstone (soft delete).
Deleted items are skipped during search and are physically removed during rebuild or snapshot re-save.

**How big can the index get?**  
It is limited by your process memory (RAM).
For larger or persistent deployments, consider using a higher-level system like [VCAL Server](https://vcal-project.com/vcal-server/).

**Does it support SIMD or hardware acceleration?**  
No. As of v0.1.2, the library is fully safe Rust (#![deny(unsafe_code)]) and does not use SIMD.
This prioritizes portability, safety, and predictable behavior across platforms.

**Batch search?**  
`batch_search(&[&[f32]], k)` allows multiple queries in a single call, reducing per-query overhead.
