use crate::{
    errors::{Result, VcalError},
    graph::Graph,
    math::Metric,
    node::{Node, NodeId},
    Hnsw,
};
use std::sync::atomic::Ordering;

const SNAPSHOT_VERSION: u32 = 1;

fn default_snapshot_version() -> u32 {
    SNAPSHOT_VERSION
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerNode {
    ext_id: u64,
    vec: Vec<f32>,
    links: Vec<Vec<NodeId>>,
    #[serde(default)]
    last_hit: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerGraph {
    nodes: Vec<SerNode>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SerIndex {
    #[serde(default = "default_snapshot_version")]
    version: u32,
    dims: usize,
    m: usize,
    ef: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    efc: Option<usize>,
    graph: SerGraph,
}

pub fn to_bytes<M: Metric>(idx: &Hnsw<M>) -> Result<Vec<u8>> {
    let nodes: Vec<SerNode> = idx
        .graph
        .nodes
        .iter()
        .filter(|n| !n.is_deleted())
        .map(|n| SerNode {
            ext_id: n.ext_id,
            vec: n.vec.clone(),
            links: n.links.clone(),
            last_hit: Some(n.last_hit.load(Ordering::Relaxed)),
        })
        .collect();

    let ser = SerIndex {
        version: SNAPSHOT_VERSION,
        dims: idx.dims,
        m: idx.m,
        ef: idx.ef,
        efc: Some(idx.efc),
        graph: SerGraph { nodes },
    };

    serde_json::to_vec(&ser).map_err(|e| VcalError::Serialize(e.to_string()))
}

pub fn from_slice<M: Metric + Default>(bytes: &[u8]) -> Result<Hnsw<M>> {
    let snap: SerIndex =
        serde_json::from_slice(bytes).map_err(|e| VcalError::Serialize(e.to_string()))?;

    if snap.version != SNAPSHOT_VERSION {
        return Err(VcalError::CorruptSnapshot(format!(
            "unsupported snapshot version: {}",
            snap.version
        )));
    }

    let efc = snap.efc.unwrap_or_else(|| snap.ef.max(1));
    let ef = snap.ef.max(1);

    let mut g = Graph::new();
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

    for sn in snap.graph.nodes.into_iter() {
        let level = sn.links.len().saturating_sub(1);
        let node_id = g.nodes.len() as NodeId;

        let mut node = Node::new(sn.ext_id, level, sn.vec);
        node.links = sn.links;
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

    g.max_level = max_level;
    g.entry = if max_level < g.levels.len() && !g.levels[max_level].is_empty() {
        Some(g.levels[max_level][0])
    } else if !g.nodes.is_empty() {
        Some(0)
    } else {
        None
    };

    let mut h = Hnsw {
        dims: snap.dims,
        m: snap.m,
        ef,
        efc,
        metric: M::default(),
        graph: g,
    };

    let _ = h.graph.sanitize();
    Ok(h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cosine, HnswBuilder};

    #[test]
    fn snapshot_roundtrip() {
        let mut h = HnswBuilder::<Cosine>::default().dims(8).build().unwrap();
        h.insert(vec![0.5; 8], 7).unwrap();

        let bytes = h.to_bytes().unwrap();
        let h2 = Hnsw::<Cosine>::from_slice(&bytes).unwrap();

        assert_eq!(h2.search(&[0.5; 8], 1).unwrap()[0].0, 7);
    }

    #[test]
    fn snapshot_bad_json_returns_error() {
        let err = Hnsw::<Cosine>::from_slice(br#"{"not":"valid enough"}"#);

        match err {
            Err(VcalError::Serialize(_)) => {}
            Err(other) => panic!("unexpected error: {}", other),
            Ok(_) => panic!("expected error"),
        }
    }

    #[test]
    fn snapshot_unknown_version_rejected() {
        let bytes = br#"{
            "version": 999,
            "dims": 8,
            "m": 16,
            "ef": 32,
            "graph": {"nodes": []}
        }"#;

        let err = Hnsw::<Cosine>::from_slice(bytes);

        match err {
            Err(VcalError::CorruptSnapshot(_)) => {}
            Err(other) => panic!("unexpected error: {}", other),
            Ok(_) => panic!("expected error"),
        }
    }
}
