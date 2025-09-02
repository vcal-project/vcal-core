//! (feature = "serde") snapshot helpers for VCAL-core.
//!
//! We **do not** serialize a `level` field. Instead, the node's level is
//! **derived** as `links.len() - 1`. This avoids keeping a separate field and
//! removes warnings about unused `level`s in the core structs.

use crate::{
    errors::{Result, VcalError},
    graph::Graph,
    math::Metric,
    node::{Node, NodeId},
    Hnsw,
};
use std::sync::atomic::Ordering;

// Use fully-qualified derives to avoid scope issues
#[derive(serde::Serialize, serde::Deserialize)]
struct SerNode {
    ext_id: u64,
    vec: Vec<f32>,
    links: Vec<Vec<NodeId>>,
    #[serde(default)]          // backward-compatible: absent in old snapshots
    last_hit: Option<u64>,     // unix seconds of last access
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerGraph {
    nodes: Vec<SerNode>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerIndex {
    dims: usize,
    m: usize,
    ef: usize,
    graph: SerGraph,
}

/// Serialize the index into JSON bytes (single file).
/// Panics only on OOM / serialization failure.
pub fn to_bytes<M: Metric>(idx: &Hnsw<M>) -> Vec<u8> {
    let nodes: Vec<SerNode> = idx
        .graph
        .nodes
        .iter()
        .filter(|n| !n.is_deleted()) // do not persist logically deleted nodes
        .map(|n| SerNode {
            ext_id: n.ext_id,
            vec: n.vec.clone(),
            links: n.links.clone(),
            last_hit: Some(n.last_hit.load(Ordering::Relaxed)),
        })
        .collect();

    let ser = SerIndex {
        dims: idx.dims,
        m: idx.m,
        ef: idx.ef,
        graph: SerGraph { nodes },
    };

    serde_json::to_vec(&ser).expect("serialize snapshot")
}

/// Restore an index from JSON bytes produced by `to_bytes`.
///
/// *The node's level is computed as `links.len().saturating_sub(1)`*.
pub fn from_slice<M: Metric + Default>(bytes: &[u8]) -> Result<Hnsw<M>> {
    let snap: SerIndex =
        serde_json::from_slice(bytes).map_err(|e| VcalError::Serialize(e.to_string()))?;

    // Build a fresh graph
    let mut g = Graph::new();

    // Determine max level so we can size `g.levels`
    let mut max_level = 0usize;
    for sn in &snap.graph.nodes {
        if sn.vec.len() != snap.dims {
            return Err(VcalError::DimensionMismatch {
                expected: snap.dims,
                found: sn.vec.len(),
            });
        }
        let level = sn.links.len().saturating_sub(1);
        if level > max_level {
            max_level = level;
        }
    }
    while g.levels.len() <= max_level {
        g.levels.push(Vec::new());
    }

    // Recreate nodes and register them per level; rebuild by_ext/active/bytes
    for sn in snap.graph.nodes.into_iter() {
        let level = sn.links.len().saturating_sub(1);
        let node_id = g.nodes.len() as NodeId;

        let mut node = Node::new(sn.ext_id, level, sn.vec);
        node.links = sn.links; // restore per-level adjacency
        if let Some(ts) = sn.last_hit {
            node.last_hit.store(ts, Ordering::Relaxed);
        }
        node.recompute_bytes();

        g.total_bytes += node.bytes;
        g.active += 1;
        g.by_ext.insert(node.ext_id, node_id);

        g.nodes.push(node);
        g.levels[level].push(node_id);
    }

    // Pick an entry point: first node on the highest level (if any),
    // else fallback to 0 for non-empty graphs.
    g.max_level = max_level;
    g.entry = if max_level < g.levels.len() && !g.levels[max_level].is_empty() {
        Some(g.levels[max_level][0])
    } else if !g.nodes.is_empty() {
        Some(0)
    } else {
        None
    };

    Ok(Hnsw {
        dims: snap.dims,
        m: snap.m,
        ef: snap.ef,
        metric: M::default(),
        graph: g,
    })
}
