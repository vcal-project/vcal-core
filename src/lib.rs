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
        Ok(self.graph.knn(query, k, &self.metric, self.ef))
    }

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
