//! params.rs — builder for configuring an `Hnsw<M>` instance.
//!
//! We keep it simple (no `thiserror` or `derive_builder`) to stay MSRV 1.56-friendly.

use crate::{
    graph::Graph,
    math::{Cosine, Metric},
    Hnsw,
};

/// Reasonable defaults from the HNSW paper (Malkov, 2018).
const DEF_M: usize              = 16;
const DEF_EF_CONSTRUCTION: usize = 200;
const DEF_EF_SEARCH: usize       = 128;

pub struct HnswBuilder<M: Metric = Cosine> {
    dims:             Option<usize>,
    m:                usize,
    ef_construction:  usize,
    ef_search:        usize,
    metric:           M,
}

impl<M: Metric> HnswBuilder<M> {
    #[must_use]
    pub fn new(metric: M) -> Self {
        Self {
            dims: None,
            m: DEF_M,
            ef_construction: DEF_EF_CONSTRUCTION,
            ef_search: DEF_EF_SEARCH,
            metric,
        }
    }

    #[inline]
    #[must_use]
    pub fn dims(mut self, d: usize) -> Self {
        self.dims = Some(d);
        self
    }

    #[inline]
    #[must_use]
    pub fn m(mut self, m: usize) -> Self {
        self.m = m.max(2);
        self
    }

    #[inline]
    #[must_use]
    pub fn ef_construction(mut self, ef: usize) -> Self {
        self.ef_construction = ef.max(1);
        self
    }

    #[inline]
    #[must_use]
    pub fn ef_search(mut self, ef: usize) -> Self {
        self.ef_search = ef.max(1);
        self
    }

    #[inline]
    #[must_use]
    pub fn metric<T: Metric>(self, metric: T) -> HnswBuilder<T> {
        HnswBuilder {
            dims: self.dims,
            m: self.m,
            ef_construction: self.ef_construction,
            ef_search: self.ef_search,
            metric,
        }
    }

    #[must_use]
    pub fn build(self) -> Hnsw<M> {
        let dims = self.dims.unwrap_or(0);
        debug_assert!(dims > 0, "HnswBuilder: call .dims() before build()");
        Hnsw {
            dims,
            m:  self.m,
            ef: self.ef_search,
            efc: self.ef_construction,
            metric: self.metric,
            graph: Graph::new(),
        }
    }
}

impl Default for HnswBuilder<Cosine> {
    fn default() -> Self {
        Self::new(Cosine)
    }
}
