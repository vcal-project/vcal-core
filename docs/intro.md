---
id: intro
title: What is vcal-core?
slug: /
sidebar_label: Introduction
---

**vcal-core** is a lightweight Rust library that provides a fast **semantic cache / vector index** built on HNSW.  
Use it to **store embeddings, search nearest neighbors**, and implement cache-like reuse in your LLM apps.

#### Highlights
- HNSW index (Cosine / Dot)
- Insert / Upsert / Delete (soft tombstone)
- TTL and capacity-based eviction (optional)
- Batch search
- Snapshots (save/load with `serde`)
- Optional SIMD acceleration

#### When to use vcal-core
- You want an **in-process** vector index with zero network hops.
- You need predictable performance and control over lifecycle (TTL/eviction).
- You don’t need a distributed vector DB for this component.

See **Quickstart** → minimal example in Rust.
