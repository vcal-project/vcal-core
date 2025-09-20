---
id: metrics
title: Metrics (Library)
sidebar_label: Metrics
---

`vcal-core` does not expose Prometheus metrics directly (that lives in `vcal-server`).  
You can, however, instrument library calls in your app:

- **Count** inserts, upserts, deletes
- **Gauge** active size (`idx.size()`)
- **Histogram** search latency (wrap `search()` with a timer)

Example (pseudo):

```rust
let t0 = std::time::Instant::now();
let res = idx.search(query, 8)?;
metrics::histogram!("vcal_core_search_ms", t0.elapsed().as_millis() as f64);
```