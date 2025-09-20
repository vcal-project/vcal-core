---
id: env
title: Environment variables

sidebar_label: Environment variables
---

## VCAL — Example .env for apps using vcal-core (library)

vcal-core itself does not read env vars; your app (or vcal-server) should load them via a dotenv loader.

## --- Embedding space ---
### One of: cosine | dot
```
VCAL_METRIC=cosine
VCAL_DIMS=768
```

## --- Capacity & TTL (optional) ---
### Max active vectors; 0 or empty = unlimited
```
VCAL_MAX_CAPACITY=200000
```
> 0 = no TTL; set >0 to expire items

```
VCAL_TTL_SECS=0
```
> How often eviction worker runs (seconds)

```
VCAL_EVICT_INTERVAL_SECS=2
```

## --- Snapshots (optional) ---
```
VCAL_SNAPSHOT_PATH=./data/vcal.snapshot
```
> Autosave every N seconds; 0 = disabled

```
VCAL_SNAPSHOT_AUTOSAVE_SECS=60
```
> 1 = atomic write (tmp+rename), 0 = direct
```
VCAL_AUTOSAVE_ATOMIC=1
```

## --- Search tuning (optional; names may differ in your app/server) ---
```
VCAL_K=8
VCAL_EF_SEARCH=128
```

## --- Observability (app/server) ---

```
RUST_LOG=info
RUST_BACKTRACE=1
```

## --- HTTP server (if using vcal-server; safe to remove for library-only) ---
```
VCAL_BIND=0.0.0.0:8080
```

## --- Cost modeling (server dashboards; optional) ---
```
VCAL_TOKENS_PER_HIT=800
```

## Notes:
- Library-only apps: read these values in your own config loader and pass to vcal-core constructors.
- vcal-server reuses many of the same names; contact support to get access to server docs to check the exact behavior.
