//! VCAL-core — minimal in-process HNSW vector index.
//!
//! * Library MSRV 1.56  (edition 2021)
//! • Dev-dependencies/benches may require a newer stable toolchain.
//! * Optional AVX2 fast-path behind `--features simd`  
//! * Optional snapshot via `serde` feature
//!
//! ## Quick-start
//! ```rust
//! use vcal_core::{HnswBuilder, Cosine};
//!
//! let mut h = HnswBuilder::<Cosine>::default()
//!     .dims(16)
//!     .build();
//!
//! h.insert(vec![1.0; 16], 42).unwrap();
//! let hits = h.search(&vec![1.0; 16], 1).unwrap();
//! assert_eq!(hits[0].0, 42);
//! ```
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod errors;
mod graph;
mod math;
mod node;
mod params;
mod rand_level;

#[cfg(feature = "serde")]
mod serialize;

#[cfg(feature = "serde")]
pub use serialize::{from_slice, to_bytes};

pub use errors::{Result, VcalError};
pub use math::{Cosine, Dot};
pub use params::HnswBuilder;

pub use rand_level::draw_level;

/// Public identifier type attached to each vector.
pub type ExternalId = u64;
/// `(id, distance)` tuple returned by `search`.
pub type SearchHit = (ExternalId, f32);

/// Main index structure.
pub struct Hnsw<M: math::Metric = math::Cosine> {
    pub(crate) dims:   usize,
    pub(crate) m:      usize,
    pub(crate) ef:     usize,
    pub(crate) efc:    usize,
    pub(crate) metric: M,
    pub(crate) graph:  graph::Graph,
}

impl<M: math::Metric> Hnsw<M> {
    /// k-NN search with a per-request `ef` override.
    /// - `k`: number of neighbors to return (pass k_expand if you overfetch upstream)
    /// - `ef`: beam width (will be clamped to at least `k` and 1)
    #[inline]
    pub fn search_with_ef(&self, query: &[f32], k: usize, ef: usize) -> Result<Vec<SearchHit>> {
        if self.graph.nodes.is_empty() {
            return Err(VcalError::EmptyIndex);
        }
        if query.len() != self.dims {
            return Err(VcalError::DimensionMismatch { expected: self.dims, found: query.len() });
        }
        // Ensure ef is sane: at least k and >=1
        let ef_eff = ef.max(k.max(1));

        let hits = self.graph.knn(query, k, &self.metric, ef_eff);

        // Feed LRU without a write-lock (same as `search`)
        let mut ids: Vec<u64> = Vec::with_capacity(hits.len());
        for (eid, _dist) in &hits { ids.push(*eid); }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.graph.touch_many(&ids, now);

        Ok(hits)
    }

    /// Return the embedding dimensionality this index was built for.
    #[inline] pub fn dims(&self) -> usize { self.dims }

    /// Set query-time ef
    #[inline]
    pub fn set_ef(&mut self, ef: usize) { self.ef = ef.max(1); }

    /// (optional) expose common params for tooling
    #[inline] pub fn params(&self) -> (usize, usize) {
        (self.m, self.ef)
    }

    #[inline]
    pub fn set_ef_construction(&mut self, efc: usize) { self.efc = efc.max(1); }

    /// Insert a vector with an external identifier.
    pub fn insert(&mut self, vec: Vec<f32>, ext_id: ExternalId) -> Result<()> {
        if vec.len() != self.dims {
            return Err(VcalError::DimensionMismatch {
                expected: self.dims,
                found: vec.len(),
            });
        }
        self.graph.add(vec, ext_id, &self.metric, self.m, self.efc);
        Ok(())
    }

    #[inline]
    pub fn params_full(&self) -> (usize, usize, usize) { (self.m, self.ef, self.efc) }

    /// k-NN search using the index’s default `ef`.
    #[inline]
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchHit>> {
        self.search_with_ef(query, k, self.ef)
    }

    /// Expose basic stats for eviction/monitoring.
    #[inline]
    pub fn stats(&self) -> (usize, usize) {
        self.graph.stats()
    }

    /// Evict by LRU until caps are satisfied (soft cap helper).
    pub fn evict_lru_until(
        &mut self,
        max_vecs: Option<usize>,
        max_bytes: Option<usize>,
    ) -> (usize, usize) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs();
        self.graph.evict_lru_until(max_vecs, max_bytes, now)
    }

    /// Idempotent delete by external id. Returns true if something was removed.
    #[inline]
    pub fn delete(&mut self, ext_id: ExternalId) -> bool {
        self.graph.delete(ext_id)
    }

    /// Check whether an id exists.
    #[inline]
    pub fn contains(&self, ext_id: ExternalId) -> bool {
        self.graph.contains_ext(ext_id)
    }

    /// TTL sweep: evict nodes whose last_hit is older than `ttl_secs`.
    #[inline]
    pub fn evict_ttl(&mut self, ttl_secs: u64) -> (usize, usize) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.graph.evict_ttl(ttl_secs, now)
    }

    /// Convenience: number of active vectors.
    #[inline]
    pub fn len(&self) -> usize { self.stats().0 }

    /// Convenience: approximate total bytes of active nodes.
    #[inline]
    pub fn total_bytes(&self) -> usize { self.stats().1 }


    // ------------------------------------------------------------------
    // Snapshot helpers (enabled with `serde`)
    // ------------------------------------------------------------------
    #[cfg(feature = "serde")]
    /// Serialise index to bytes (`serde_json` by default).
    /// Note: `vcal_core::to_bytes(&hnsw)` is also available as a free function.
    pub fn to_bytes(&self) -> Vec<u8> {
        serialize::to_bytes(self)
    }

    #[cfg(feature = "serde")]
    /// Restore index from bytes.
    /// Note: `vcal_core::from_slice::<M>(bytes)` is also available as a free function.
    pub fn from_slice(bytes: &[u8]) -> Result<Self>
    where
        M: Default,
    {
        // 1) deserialize the whole index (Self), not the metric M
        let mut h: Self = serialize::from_slice::<Self>(bytes)?;

        // 2) auto-repair any minor inconsistencies in the graph
        let (edges, nodes) = h.graph_mut().sanitize();
        if edges > 0 || nodes > 0 {
            log::warn!(
                "Sanitized snapshot: dropped {} edges, fixed {} nodes",
                edges, nodes
            );
        }
        Ok(h)
    }
}

// ----------------------------------------------------------------------
// Smoke-tests (compile-time verified)
// ----------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_insert_search() {
        let mut h = HnswBuilder::<Cosine>::default().dims(16).build();
        h.insert(vec![1.0; 16], 1).unwrap();
        let res = h.search(&vec![1.0; 16], 1).unwrap();
        assert_eq!(res[0].0, 1);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn snapshot_roundtrip() {
        let mut h = HnswBuilder::<Cosine>::default().dims(8).build();
        h.insert(vec![0.5; 8], 7).unwrap();
        let bytes = h.to_bytes();
        let h2 = Hnsw::<Cosine>::from_slice(&bytes).unwrap();
        assert_eq!(h2.search(&vec![0.5; 8], 1).unwrap()[0].0, 7);
    }

    #[test]
    fn search_with_ef_compiles_and_runs() {
        let mut h = HnswBuilder::<Cosine>::default().dims(8).ef_search(8).build();
        h.insert(vec![1.0; 8], 1).unwrap();
        let hits = h.search_with_ef(&vec![1.0; 8], 1, 32).unwrap();
        assert_eq!(hits[0].0, 1);
    }

    #[test]
    fn search_k_zero_returns_empty() {
        let mut h = HnswBuilder::<Cosine>::default().dims(4).build();
        h.insert(vec![1.0;4], 1).unwrap();
        let hits = h.search_with_ef(&[1.0;4], 0, 8).unwrap();
        assert!(hits.is_empty());
    }
}
