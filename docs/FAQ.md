# ❓ VCAL Pro — Frequently Asked Questions (FAQ)

---

## General

**Q: What is VCAL?**  
VCAL (Vector Cache-as-a-Library) is a semantic cache built on HNSW (Hierarchical Navigable Small World graphs).  
It accelerates LLM-based applications by **caching embeddings and responses**, cutting repeated LLM calls, latency, and costs.

---

**Q: Who is VCAL for?**  
- **CIOs / CTOs:** lower infrastructure cost, predictable latency, in-house control.  
- **DevOps / MLOps:** easy deployment, Prometheus metrics, runs anywhere (bare metal, containers, Kubernetes).  
- **Developers:** a lightweight library (Rust crate) and a Pro server with HTTP/JSON API.  
- **Executives (CEO / CFO):** direct reduction in cloud API bills, improved user experience, no vendor lock-in.

---

**Q: Is VCAL an alternative to a vector database?**  
Not exactly. VCAL is an **embedded semantic cache**:
- In-process, small-scale, low-latency.
- Designed to cache and deduplicate repeated queries (LLM prompts).  
- Not a general-purpose vector DB for billions of vectors and analytical queries.  
Think of it as **Redis for embeddings**, not Snowflake.

---

## Technical

**Q: How does VCAL scale?**  
- Ideal range: up to a few million vectors per instance.  
- Multi-instance scaling possible behind a load-balancer.  
- Eviction policies (TTL + LRU) ensure memory remains bounded.  
- Observability (Prometheus) supports autoscaling and alerting.

---

**Q: Does VCAL support deletes and updates?**  
Yes.  
- **Insert** adds a new vector.  
- **Upsert** replaces if exists.  
- **Delete** marks as tombstone and removes from index if present.  
All operations are idempotent.

---

**Q: What persistence options exist?**  
- Snapshots can be saved to disk via API (`/v1/snapshot/save`).  
- Server can load snapshots on startup (`VCAL_SNAPSHOT` or config).  
- Snapshots are JSON-based; atomic writes supported.  
- Intended for fast restart, not long-term archival.

---

**Q: What about eviction?**  
- Configurable **TTL eviction** (time-based).  
- **LRU eviction** based on vector/byte capacity.  
- Background task runs every `VCAL_EVICT_INTERVAL_SECS` (default 60s).  

---

**Q: How is it configured?**  
- Primary config file: `vcal.toml` (e.g., `/etc/vcal/vcal.toml`).  
- All options overridable via environment (`VCAL_*`).  
- Env always overrides TOML for flexibility (CI/CD, containers, Helm charts).

---

**Q: What observability is included?**  
Native **Prometheus metrics**:  
- Counters for requests, errors, cache hits/misses.  
- Gauges for active IDs, snapshot unixtime.  
- Histograms for search latency.  
Metrics integrate easily with Grafana dashboards.

---

**Q: How secure is it?**  
- Runs entirely on your infrastructure (no external calls).  
- No API keys or auth by default — designed to be deployed **inside a trusted network**.  
- Can be secured behind reverse proxies, mTLS, or service meshes (e.g., Istio, Nginx, Envoy).

---

## Business

**Q: What business value does VCAL deliver?**  
- **Cost savings:** fewer LLM API calls → lower OpenAI/Anthropic costs.  
- **Performance:** faster responses → better UX and productivity.  
- **Control:** runs on-prem or in your cloud → data stays with you.  
- **Simplicity:** easy integration (HTTP API, Rust crate).  

---

**Q: Does making VCAL public (open-source) harm its business model?**  
No.  
- **Core (Apache-2.0):** always open for trust and adoption.  
- **Pro server (private license/EULA):** adds enterprise-grade features (observability, eviction, snapshots).  
This model follows the successful playbook of Elastic, Redis, and HashiCorp.

---

**Q: Can VCAL be integrated into existing stacks?**  
Yes.  
- As a **library** (`vcal-core`) inside Rust services.  
- As a **standalone HTTP server** (`vcal-server`) accessible from Python, Node.js, Java, Go, etc.  
- Supports Kubernetes deployment with ConfigMaps, Secrets, and Prometheus scraping.

---

**Q: How do we try it out?**  
- Clone the repo and run locally:
  ```bash
  VCAL_DIMS=128 cargo run -p vcal-server
  ```
- Insert vectors via `curl /v1/insert`.  
- Query with `curl /v1/search`.  
- Explore metrics via `curl /metrics`.  

---

**Q: What’s the roadmap?**  
Planned features:  
- LangChain / LlamaIndex adapters.  
- WASM/WASI packaging.  
- CLI utilities.  
- Advanced snapshot formats (binary).  
- Optional authentication layer.
