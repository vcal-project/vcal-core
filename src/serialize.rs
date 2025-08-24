//! (feature = "serde") snapshot helpers for VCAL-core.
//!
//! We **do not** serialize a `level` field. Instead, the node's level is
//! **derived** as `links.len() - 1`. This avoids keeping a separate field and
//! removes warnings about unused `level`s in the core structs.

use serde::{Deserialize, Serialize};

use crate::{
    errors::{Result, VcalError},
    graph::Graph,
    math::Metric,
    node::{Node, NodeId},
    Hnsw,
};

#[derive(Serialize, Deserialize)]
struct SerNode {
    ext_id: u64,
    vec: Vec<f32>,
    links: Vec<Vec<NodeId>>,
}

#[derive(Serialize, Deserialize)]
struct SerGraph {
    nodes: Vec<SerNode>,
}

#[derive(Serialize, Deserialize)]
struct SerIndex {
    dims: usize,
    m: usize,
    ef: usize,
    graph: SerGraph,
}

/// Serialize the index into JSON bytes (human-readable single file).
/// Panics only on OOM (serde_json failure).
pub fn to_bytes<M: Metric>(idx: &Hnsw<M>) -> Vec<u8> {
    // We do not store an explicit "level" field; it can be derived from links.len().
    let nodes: Vec<SerNode> = idx
        .graph
        .nodes
        .iter()
        .map(|n| SerNode {
            ext_id: n.ext_id,
            vec: n.vec.clone(),
            links: n.links.clone(),
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
        serde_json::from_slice(bytes).map_err(|_| VcalError::Serialize)?;

    // Build a fresh graph
    let mut g = Graph::new();

    // Ensure levels capacity based on the maximum observed link depth
    let mut max_level = 0usize;

    // First pass: determine max level so we can size `g.levels`
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

    // Second pass: recreate nodes and register them per level
    for sn in snap.graph.nodes.into_iter() {
        let level = sn.links.len().saturating_sub(1);
        let node_id = g.nodes.len() as NodeId;

        // Construct node with derived level; then override links snapshot
        let mut node = Node::new(node_id, sn.ext_id, level, sn.vec);
        node.links = sn.links;

        g.nodes.push(node);
        g.levels[level].push(node_id);
    }

    // Pick an entry point: the first node on the highest level (if any),
    // otherwise fall back to 0 for non-empty graphs.
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
