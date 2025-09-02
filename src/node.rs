//! node.rs — node definition and helpers for VCAL-core

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// A node owns its vector and adjacency lists for each HNSW level.
// Links are stored in `SmallVec` to keep most nodes on-stack (fast cache).

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
    // --- LRU/TTL metadata ---
    pub(crate) last_hit: AtomicU64,   // unix secs of last access
    pub(crate) deleted:  AtomicBool,  // true if evicted/deleted
    pub(crate) bytes:    usize,       // rough footprint for caps
}

impl Node {
    pub fn new(ext_id: u64, level: usize, vec: Vec<f32>) -> Self {
        let mut links = Vec::with_capacity(level + 1);
        for _ in 0..=level {
            links.push(Vec::new());
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Initialize with bytes = 0, then compute from actual lengths.
        let mut s = Self {
            ext_id,
            vec,
            links,
            last_hit: AtomicU64::new(now),
            deleted: AtomicBool::new(false),
            bytes: 0,
        };
        s.recompute_bytes();
        s
    }

    /// Touch the node (mark as recently used).
    #[inline]
    pub fn touch(&self, now_unix: u64) {
        self.last_hit.store(now_unix, Ordering::Relaxed);
    }

    #[inline]
    pub fn is_deleted(&self) -> bool {
        self.deleted.load(Ordering::Relaxed)
    }

    /// Recompute this node's approximate byte footprint.
    /// Call after adding/removing links or changing the vector.
    #[inline]
    pub fn recompute_bytes(&mut self) -> usize {
        let mut b = self.vec.len() * std::mem::size_of::<f32>();
        for l in &self.links {
            b += l.len() * std::mem::size_of::<NodeId>();
        }
        self.bytes = b;
        b
    }
}
