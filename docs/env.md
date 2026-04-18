---
id: env
title: Environment variables

sidebar_label: Environment variables
---

## VCAL — Example .env (for apps using vcal-core)

**vcal-core** does not read environment variables directly.
Use these as conventions in your own application config layer.  

To learn more about VCAL Server contact see [here](https://vcal-project.com/vcal-server).

## --- Embedding space ---
```env
VCAL_METRIC=cosine   # cosine | dot
VCAL_DIMS=768
```

> Map `cosine` → `Cosine`, `dot` → `Dot` when constructing the index.

## --- Capacity & TTL (application-defined)
```
VCAL_MAX_CAPACITY=200000   # 0 or unset = unlimited
VCAL_TTL_SECS=0            # 0 = no TTL
VCAL_EVICT_INTERVAL_SECS=2 # how often your eviction logic runs
```
> vcal-core provides TTL and eviction primitives, but you control scheduling and enforcement.

```
VCAL_TTL_SECS=0
```
> How often eviction worker runs (seconds)

```
VCAL_EVICT_INTERVAL_SECS=2
```

## --- Snapshots (application-defined) ---
```
VCAL_SNAPSHOT_PATH=./data/vcal.snapshot
VCAL_SNAPSHOT_AUTOSAVE_SECS=60
VCAL_AUTOSAVE_ATOMIC=1
```
> Snapshot persistence is manual in vcal-core (`to_bytes` / `from_slice`).
> Autosave, atomic writes, and scheduling must be implemented in your application.

## --- Search tuning ---
```
VCAL_K=8
VCAL_EF_SEARCH=128
```

> These map to runtime parameters such as `search(k)` and builder configuration.

## --- Observability ---

```
RUST_LOG=info
RUST_BACKTRACE=1
```

## --- Optional: VCAL Server settings ---
```
VCAL_BIND=0.0.0.0:8080
VCAL_TOKENS_PER_HIT=800
```

> These apply only when running VCAL Server.

## Notes:
- Treat this file as a reference config, not a built-in interface
- Load values via your preferred config system (`dotenv`, `envy`, etc.)
- Pass parsed values into:
  - `HnswBuilder`
  - your eviction logic
  - your snapshot scheduler
