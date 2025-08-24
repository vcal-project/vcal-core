//! node.rs — node definition and helpers for VCAL-core
//!
//! A node owns its vector and adjacency lists for each HNSW level.
//! Links are stored in `SmallVec` to keep most nodes on-stack (fast cache).

/// Internal graph index (usize is fine; usize → u32 later if we quantise)
pub type NodeId = usize;

/// Maximum links we expect to keep per level (`M` parameter).  
/// Adjust if you change the builder default.
pub(crate) const MAX_LINKS_PER_LVL: usize = 32;

/// One vector in the HNSW graph.
pub struct Node {
    pub(crate) ext_id: u64,
    pub(crate) vec: Vec<f32>,
    pub(crate) links: Vec<Vec<NodeId>>, // per-level adjacency
}

impl Node {
    pub fn new(ext_id: u64, level: usize, vec: Vec<f32>) -> Self {
        let mut links = Vec::with_capacity(level + 1);
        for _ in 0..=level { links.push(Vec::new()); }
        Self { ext_id, vec, links }
    }
}