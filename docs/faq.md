---
id: faq
title: FAQ

sidebar_label: FAQ
---

**Is vcal-core a database?**  
No. It’s an **in-process** vector index library. Embed it in your service.

**Cosine vs Dot?**  
Use **Cosine** for typical text embeddings. Dot is fine if your embedding pipeline expects it.

**Can I delete items?**  
Yes: `delete(id)` sets a tombstone; index maintenance will drop it on rebuild/snapshot.

**How big can the index get?**  
Constrained by your process memory. Start with capacity planning + snapshots.

**Does it support MIPS/ARM/SIMD?**  
Yes where available (enable the `simd` feature). Fallback paths exist.

**Batch search?**  
`batch_search(&[&[f32]], k)` is provided to reduce per-query overhead.
