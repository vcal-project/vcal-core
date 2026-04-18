---
id: intro
title: What is vcal-core?
slug: /
sidebar_label: Introduction
---

**vcal-core** is a lightweight Rust library for building fast **semantic caches and vector indexes** using HNSW.
It lets you **store embeddings, perform nearest-neighbor search**, and implement cache-like reuse in LLM applications.

#### Highlights
- HNSW index (Cosine / Dot)
- Insert / Upsert / Delete (soft tombstone)
- TTL and capacity-based eviction (optional)
- Batch search
- Snapshot support via **serde** (byte-based serialization)
- Fully safe Rust (`#![deny(unsafe_code)]`)


#### When to use vcal-core
- You want an **in-process** vector index with zero network hops
- You need predictable performance and control over lifecycle (TTL/eviction)
- You are building a semantic cache or retrieval layer inside your application
- You don’t need a distributed vector DB for this component

See **Quickstart** → minimal example in Rust.
