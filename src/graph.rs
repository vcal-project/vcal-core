//! graph.rs — core HNSW graph implementation for VCAL-core.

use crate::{
    math::Metric,
    node::{Node, NodeId},
    rand_level::draw_level,
};
use ordered_float::OrderedFloat;
use smallvec::SmallVec;
use crate::node::MAX_LINKS_PER_LVL;
use std::cmp::Reverse;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

/// In-memory HNSW graph.
pub struct Graph {
    pub nodes: Vec<Node>,
    pub(crate) levels: Vec<Vec<NodeId>>,
    pub(crate) max_level: usize,
    pub(crate) entry: Option<NodeId>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            levels: vec![Vec::new()], // ensure level-0 exists
            max_level: 0,
            entry: None,
        }
    }

    /// Safe accessor: neighbors of `nid` on `layer` (empty slice if absent).
    #[inline]
    fn neighbors(&self, nid: NodeId, layer: usize) -> &[NodeId] {
        if nid >= self.nodes.len() {
            return &[];
        }
        let links = &self.nodes[nid].links;
        if layer >= links.len() {
            return &[];
        }
        &links[layer]
    }

    /// Insert a vector + external id.
    pub fn add<M: Metric>(
        &mut self,
        vec: Vec<f32>,
        ext_id: u64,
        metric: &M,
        m: usize,
        ef: usize,
    ) {
        let lvl = draw_level(m as f64);
        let node_id = self.nodes.len() as NodeId;

        // Use the existing tower for wiring; update top only after linking.
        let old_max = self.max_level;
        let old_entry = self.entry;

        if lvl > old_max {
            for _ in old_max + 1..=lvl {
                self.levels.push(Vec::new());
            }
        }

        self.nodes.push(Node::new(ext_id, lvl, vec));

        let mut entry = old_entry.unwrap_or(node_id);

        if node_id != entry && old_max >= lvl + 1 {
            for l in (lvl + 1..=old_max).rev() {
                entry = self.greedy(entry, node_id, l, metric);
            }
        }

        for l in (0..=lvl).rev() {
            let mut neigh = self.ef_search_idx(entry, &self.nodes[node_id].vec, ef, l, metric);
            neigh.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

            let mut ids = Vec::with_capacity(neigh.len());
            for (nid2, _) in neigh {
                if nid2 != node_id {
                    ids.push(nid2);
                }
            }
            if ids.is_empty() && entry != node_id {
                ids.push(entry);
            }
            self.connect(node_id, &ids, m, l, metric);
        }

        if lvl > old_max {
            self.max_level = lvl;
            self.entry = Some(node_id);
        }

        while self.levels.len() <= lvl {
            self.levels.push(Vec::new());
        }
        self.levels[lvl].push(node_id);
    }

    /// Public k-NN search (returns `(ext_id, dist)`).
    pub fn knn<M: Metric>(
        &self,
        query: &[f32],
        k: usize,
        metric: &M,
        ef: usize,
    ) -> Vec<(u64, f32)> {
        if self.nodes.is_empty() {
            return Vec::new();
        }
        let mut ep = self.entry.unwrap();

        for l in (1..=self.max_level).rev() {
            ep = self.greedy_idx(ep, query, l, metric);
        }

        let mut cand = self.ef_search_idx(ep, query, ef.max(k), 0, metric);
        cand.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        cand.truncate(k);
        cand.into_iter()
            .map(|(nid, dist)| (self.nodes[nid].ext_id, dist))
            .collect()
    }

    /* ---------------- internal helpers ----------------------------------- */

    fn greedy<M: Metric>(
        &self,
        mut curr: NodeId,
        target: NodeId,
        layer: usize,
        metric: &M,
    ) -> NodeId {
        if self.neighbors(curr, layer).is_empty() {
            return curr;
        }
        let tv = &self.nodes[target].vec;
        loop {
            let mut improved = false;
            for &nb in self.neighbors(curr, layer) {
                if metric.distance(&self.nodes[nb].vec, tv)
                    < metric.distance(&self.nodes[curr].vec, tv)
                {
                    curr = nb;
                    improved = true;
                }
            }
            if !improved {
                break;
            }
        }
        curr
    }

    fn greedy_idx<M: Metric>(
        &self,
        mut curr: NodeId,
        q: &[f32],
        layer: usize,
        metric: &M,
    ) -> NodeId {
        if self.neighbors(curr, layer).is_empty() {
            return curr;
        }
        loop {
            let mut improved = false;
            for &nb in self.neighbors(curr, layer) {
                if metric.distance(&self.nodes[nb].vec, q)
                    < metric.distance(&self.nodes[curr].vec, q)
                {
                    curr = nb;
                    improved = true;
                }
            }
            if !improved {
                break;
            }
        }
        curr
    }

    /// ef-search core — returns Vec of (NodeId, distance).
    fn ef_search_idx<M: Metric>(
        &self,
        entry: NodeId,
        query: &[f32],
        ef: usize,
        layer: usize,
        metric: &M,
    ) -> Vec<(NodeId, f32)> {
        let mut visited = HashSet::with_capacity(ef * 2);

        // best candidates kept as a max-heap keyed by positive distance (peek = worst)
        let mut top: BinaryHeap<(OrderedFloat<f32>, NodeId)> = BinaryHeap::new();
        // frontier is a min-heap (closest first)
        let mut to_visit: BinaryHeap<(Reverse<OrderedFloat<f32>>, NodeId)> =
            BinaryHeap::new();

        let d0 = metric.distance(&self.nodes[entry].vec, query);
        visited.insert(entry);
        top.push((OrderedFloat(d0), entry));
        to_visit.push((Reverse(OrderedFloat(d0)), entry));

        while let Some((Reverse(_), curr)) = to_visit.pop() {
            let neighs = self.neighbors(curr, layer);
            if neighs.is_empty() {
                continue;
            }
            let worst = top.peek().map(|x| x.0.into_inner()).unwrap_or(f32::INFINITY);
            for &nb in neighs {
                if !visited.insert(nb) {
                    continue;
                }
                let d = metric.distance(&self.nodes[nb].vec, query);
                if top.len() < ef || d < worst {
                    to_visit.push((Reverse(OrderedFloat(d)), nb));
                    top.push((OrderedFloat(d), nb));
                    if top.len() > ef {
                        top.pop();
                    }
                }
            }
        }

        top.into_iter().map(|(od, nid)| (nid, od.into_inner())).collect()
    }

    fn connect<M: Metric>(
        &mut self,
        nid: NodeId,
        neigh: &[NodeId],
        m: usize,
        layer: usize,
        metric: &M,
    ) {
        let mut selected = SmallVec::<[NodeId; MAX_LINKS_PER_LVL]>::new();
        for &c in neigh {
            if c == nid {
                continue; // no self-edge
            }
            if selected.len() >= m {
                break;
            }
            let ok = selected.iter().all(|&s| {
                metric.distance(&self.nodes[c].vec, &self.nodes[nid].vec)
                    < metric.distance(&self.nodes[c].vec, &self.nodes[s].vec)
            });
            if ok {
                selected.push(c);
            }
        }

        if layer < self.nodes[nid].links.len() {
            self.nodes[nid].links[layer].extend_from_slice(&selected);
        }
        for &s in &selected {
            if layer < self.nodes[s].links.len() {
                self.nodes[s].links[layer].push(nid);
            }
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
