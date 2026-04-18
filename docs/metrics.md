---
id: metrics
title: Metrics (Library)
sidebar_label: Metrics
---

`vcal-core` does not expose Prometheus metrics directly.  
Metrics and observability are provided at the service layer (for example, in `vcal-server`).

You can still instrument library calls in your own application:

- **Counter** for inserts, upserts, and deletes
- **Gauge** for approximate active size (`idx.size()`)
- **Histogram** for search latency (wrap `search()` with a timer)

Example:

```rust
let t0 = std::time::Instant::now();
let res = idx.search(query, 8)?;
metrics::histogram!("vcal_core_search_ms", t0.elapsed().as_millis() as f64);
