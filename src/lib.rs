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
    pub(crate) metric: M,
    pub(crate) graph:  graph::Graph,
}

impl<M: math::Metric> Hnsw<M> {
    /// Return the embedding dimensionality this index was built for.
    #[inline] pub fn dims(&self) -> usize { self.dims }

    /// Set query-time ef
    #[inline] pub fn set_ef(&mut self, ef: usize) { self.ef = ef; }

    /// (optional) expose common params for tooling
    #[inline] pub fn params(&self) -> (usize, usize) {
        (self.m, self.ef)
    }
    /// Insert a vector with an external identifier.
    pub fn insert(&mut self, vec: Vec<f32>, ext_id: ExternalId) -> Result<()> {
        if vec.len() != self.dims {
            return Err(VcalError::DimensionMismatch {
                expected: self.dims,
                found: vec.len(),
            });
        }
        self.graph.add(vec, ext_id, &self.metric, self.m, self.ef);
        Ok(())
    }

    /// k-nearest neighbour search.
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<SearchHit>> {
        if self.graph.nodes.is_empty() {
            return Err(VcalError::EmptyIndex);
        }
        if query.len() != self.dims {
            return Err(VcalError::DimensionMismatch {
                expected: self.dims,
                found: query.len(),
            });
        }
        let hits = self.graph.knn(query, k, &self.metric, self.ef);
        // Touch winners to feed LRU without taking a write-lock.
        #[allow(unused_mut)]
        let mut ids: Vec<u64> = Vec::with_capacity(hits.len());
        for (eid, _dist) in &hits {
            ids.push(*eid);
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.graph.touch_many(&ids, now);
        Ok(hits)
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
    pub fn to_bytes(&self) -> Vec<u8> {
        serialize::to_bytes(self)
    }

    #[cfg(feature = "serde")]
    /// Restore index from bytes.
    pub fn from_slice(bytes: &[u8]) -> Result<Self>
    where
        M: Default,
    {
        serialize::from_slice::<M>(bytes)
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
}
