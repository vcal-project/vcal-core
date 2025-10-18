//! graph.rs — core HNSW graph implementation for VCAL-core.

use crate::{
    math::Metric,
    node::{Node, NodeId},
    rand_level::draw_level,
};

use smallvec::SmallVec;
use crate::node::MAX_LINKS_PER_LVL;
use std::cmp::Ordering;
use std::collections::HashMap;

/// In-memory HNSW graph.
pub struct Graph {
    pub nodes: Vec<Node>,
    pub(crate) levels: Vec<Vec<NodeId>>,
    pub(crate) max_level: usize,
    pub(crate) entry: Option<NodeId>,
    /// Mapping from external ids to internal NodeId for fast lookups/touch/delete.
    pub(crate) by_ext: HashMap<u64, NodeId>,
    /// Active (non-deleted) vectors count.
    pub(crate) active: usize,
    /// Approximate total bytes across active nodes (guides eviction).
    pub(crate) total_bytes: usize,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            levels: vec![Vec::new()], // ensure level-0 exists
            max_level: 0,
            entry: None,
            by_ext: HashMap::new(),
            active: 0,
            total_bytes: 0,
        }
    }

    /// Safe accessor: neighbors of `nid` on `layer` (empty slice if absent).
    #[inline]
    fn neighbors(&self, nid: NodeId, layer: usize) -> &[NodeId] {
        if nid >= self.nodes.len() { return &[]; }
        let links = &self.nodes[nid].links;
        if layer >= links.len() { return &[]; }
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
        // If the external id already exists, treat as upsert: delete old node first.
        if let Some(_old) = self.by_ext.get(&ext_id).copied() {
            // Best-effort idempotent delete; ignore result.
            let _ = self.delete(ext_id);
        }

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

        let node = Node::new(ext_id, lvl, vec);
        self.total_bytes += node.bytes;
        self.active += 1;
        self.nodes.push(node);
        self.by_ext.insert(ext_id, node_id);

        let mut entry = old_entry.unwrap_or(node_id);

        if node_id != entry && old_max >= lvl + 1 {
            for l in (lvl + 1..=old_max).rev() {
                entry = self.greedy(entry, node_id, l, metric);
            }
        }

        for l in (0..=lvl).rev() {
            let ef_eff = ef.max(m.max(1));
            let mut neigh = self.ef_search_idx(entry, &self.nodes[node_id].vec, ef_eff, l, metric);

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
            ids.retain(|&x| x < self.nodes.len() && !self.nodes[x].is_deleted() && x != node_id);
            self.connect(node_id, &ids, m, l, metric);
        }

        // Ensure entry is set for the very first node.
        if self.entry.is_none() {
            self.entry = Some(node_id);
        }
        // Raise max_level if this node extends the tower.
        if lvl > old_max {
            self.max_level = lvl;
        }

        while self.levels.len() <= lvl {
            self.levels.push(Vec::new());
        }
        self.levels[lvl].push(node_id);
    }

    /// Public k-NN search (returns `(ext_id, dist)`).
    pub fn knn<M: Metric>(&self, query: &[f32], k: usize, metric: &M, ef: usize) -> Vec<(u64, f32)> {
        if self.nodes.is_empty() || k == 0 { return Vec::new(); }

        // Don’t trust self.entry blindly.
        let mut ep = match self.entry {
            Some(e) if self.is_valid_nid(e) => e,
            _ => match self.pick_entry() {
                Some(e) => e,
                None => return Vec::new(),
            },
        };

        for l in (1..=self.max_level).rev() {
            ep = self.greedy_idx(ep, query, l, metric);
        }

        let mut cand = self.ef_search_idx(ep, query, ef.max(k), 0, metric);
        cand.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        cand.truncate(k);
        cand.into_iter().map(|(nid, dist)| (self.nodes[nid].ext_id, dist)).collect()
    }

    /* ---------------- internal helpers ----------------------------------- */

    fn greedy<M: Metric>(&self, mut curr: NodeId, target: NodeId, layer: usize, metric: &M) -> NodeId {
        if !self.is_valid_nid(curr) || self.neighbors(curr, layer).is_empty() { return curr; }
        let tv = &self.nodes[target].vec;
        loop {
            let mut improved = false;
            for &nb in self.neighbors(curr, layer) {
                if !self.is_valid_nid(nb) { continue; }
                if metric.distance(&self.nodes[nb].vec, tv) < metric.distance(&self.nodes[curr].vec, tv) {
                    curr = nb; improved = true;
                }
            }
            if !improved { break; }
        }
        curr
    }

    fn greedy_idx<M: Metric>(&self, mut curr: NodeId, q: &[f32], layer: usize, metric: &M) -> NodeId {
        if !self.is_valid_nid(curr) || self.neighbors(curr, layer).is_empty() { return curr; }
        loop {
            let mut improved = false;
            for &nb in self.neighbors(curr, layer) {
                if !self.is_valid_nid(nb) { continue; }
                if metric.distance(&self.nodes[nb].vec, q) < metric.distance(&self.nodes[curr].vec, q) {
                    curr = nb; improved = true;
                }
            }
            if !improved { break; }
        }
        curr
    }

    /// ef-search core — returns Vec of (NodeId, distance).
    fn ef_search_idx<M: Metric>(
        &self, entry: NodeId, query: &[f32], ef: usize, layer: usize, metric: &M,
    ) -> Vec<(NodeId, f32)> {
        // Bail out early if entry is invalid/deleted.
        if !self.is_valid_nid(entry) { return Vec::new(); }

        let mut visited = std::collections::HashSet::with_capacity(ef * 2);
        use std::cmp::Reverse;
        use ordered_float::OrderedFloat;
        let mut top: std::collections::BinaryHeap<(OrderedFloat<f32>, NodeId)> = std::collections::BinaryHeap::new();
        let mut to_visit: std::collections::BinaryHeap<(Reverse<OrderedFloat<f32>>, NodeId)> =
            std::collections::BinaryHeap::new();

        let d0 = metric.distance(&self.nodes[entry].vec, query);
        visited.insert(entry);
        top.push((OrderedFloat(d0), entry));
        to_visit.push((Reverse(OrderedFloat(d0)), entry));

        while let Some((Reverse(_), curr)) = to_visit.pop() {
            let neighs = self.neighbors(curr, layer);
            if neighs.is_empty() { continue; }
            let worst = top.peek().map(|x| x.0.into_inner()).unwrap_or(f32::INFINITY);

            for &nb in neighs {
                if !self.is_valid_nid(nb) { continue; } // <==== extra guard
                if !visited.insert(nb) { continue; }
                let d = metric.distance(&self.nodes[nb].vec, query);
                if top.len() < ef || d < worst {
                    to_visit.push((Reverse(OrderedFloat(d)), nb));
                    top.push((OrderedFloat(d), nb));
                    if top.len() > ef { top.pop(); }
                }
            }
        }
        top.into_iter().map(|(od, nid)| (nid, od.into_inner())).collect()
    }

    /// Check whether an ext_id exists.
    pub fn contains_ext(&self, ext_id: u64) -> bool {
        self.by_ext.contains_key(&ext_id)
    }

    fn connect<M: Metric>(
        &mut self,
        nid: NodeId,
        neigh: &[NodeId],
        m: usize,
        layer: usize,
        metric: &M,
    ) {
        // --- neighbor selection (greedy HNSW heuristic) ---
        let mut selected = SmallVec::<[NodeId; MAX_LINKS_PER_LVL]>::new();
        for &c in neigh {
            if c == nid { continue; }
            if selected.len() >= m { break; }
            let ok = selected.iter().all(|&s| {
                metric.distance(&self.nodes[c].vec, &self.nodes[nid].vec)
                    < metric.distance(&self.nodes[c].vec, &self.nodes[s].vec)
            });
            if ok { selected.push(c); }
        }

        // Pre-filter: only keep valid neighbors, then do all mutations afterwards.
        let mut selected_valid = Vec::with_capacity(selected.len());
        for &s in &selected {
            if s < self.nodes.len() && !self.nodes[s].is_deleted() && s != nid {
                selected_valid.push(s);
            }
        }

        // Ensure layer exists on nid and each valid neighbor
        if layer >= self.nodes[nid].links.len() {
            self.nodes[nid].links.resize(layer + 1, Vec::new());
        }
        for &s in &selected_valid {
            if layer >= self.nodes[s].links.len() {
                self.nodes[s].links.resize(layer + 1, Vec::new());
            }
        }

        // Byte accounting
        let mut bytes_delta: isize = 0;

        // Forward (nid -> selected_valid)
        let nid_before = self.nodes[nid].recompute_bytes();
        {
            let mut adj = std::mem::take(&mut self.nodes[nid].links[layer]);
            adj.extend(selected_valid.iter().copied());
            adj.retain(|&x| x != nid && x < self.nodes.len() && !self.nodes[x].is_deleted());
            adj.sort_unstable();
            adj.dedup();
            self.nodes[nid].links[layer] = adj;
        }
        let nid_after = self.nodes[nid].recompute_bytes();
        bytes_delta += (nid_after as isize) - (nid_before as isize);

        // Back-edges (selected_valid -> nid)
        for &s in &selected_valid {
            let nb_before = self.nodes[s].recompute_bytes();
            {
                let mut adj = std::mem::take(&mut self.nodes[s].links[layer]);
                adj.push(nid);
                adj.retain(|&x| x != s && x < self.nodes.len() && !self.nodes[x].is_deleted());
                adj.sort_unstable();
                adj.dedup();
                self.nodes[s].links[layer] = adj;
            }
            self.prune_degree_hnsw(s, layer, m, metric);
            let nb_after = self.nodes[s].recompute_bytes();
            bytes_delta += (nb_after as isize) - (nb_before as isize);
        }

        // Prune nid as well
        self.prune_degree_hnsw(nid, layer, m, metric);

        // Apply accumulated delta
        if bytes_delta >= 0 {
            self.total_bytes = self.total_bytes.saturating_add(bytes_delta as usize);
        } else {
            self.total_bytes = self.total_bytes.saturating_sub((-bytes_delta) as usize);
        }
    }

    /// Greedy HNSW degree pruning: keep up to m neighbors on `layer`.
    fn prune_degree_hnsw<M: Metric>(
        &mut self,
        nid: NodeId,
        layer: usize,
        m: usize,
        metric: &M,
    ) {
        let adj_taken = std::mem::take(&mut self.nodes[nid].links[layer]);
        if adj_taken.len() <= m {
            self.nodes[nid].links[layer] = adj_taken;
            return;
        }

        // Rank by distance to nid (ascending)
        let mut cand: Vec<(NodeId, f32)> = adj_taken
            .iter()
            .copied()
            .filter(|&c| c < self.nodes.len() && !self.nodes[c].is_deleted() && c != nid)
            .map(|c| (c, metric.distance(&self.nodes[c].vec, &self.nodes[nid].vec)))
            .collect();
        cand.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Greedy keep ≤ m
        let mut keep: Vec<NodeId> = Vec::with_capacity(m);
        for (c, _) in cand {
            if keep.len() >= m { break; }
            let ok = keep.iter().all(|&s| {
                metric.distance(&self.nodes[c].vec, &self.nodes[nid].vec)
                    < metric.distance(&self.nodes[c].vec, &self.nodes[s].vec)
            });
            if ok { keep.push(c); }
        }

        self.nodes[nid].links[layer] = keep;
    }
}

impl Graph {
    #[allow(dead_code)]
    pub fn sanitize(&mut self) -> (usize, usize) {
        let mut edges_dropped = 0usize;
        let mut nodes_fixed   = 0usize;

        let nlen = self.nodes.len();
        // ✅ Precompute deletion flags to avoid immutably borrowing self.nodes while iter_mut is active
        let deleted: Vec<bool> = self.nodes.iter().map(|n| n.is_deleted()).collect();

        for (nid, n) in self.nodes.iter_mut().enumerate() {
            if n.links.is_empty() {
                n.links.resize(1, Vec::new());
                nodes_fixed += 1;
            }
            for l in 0..n.links.len() {
                let adj = &mut n.links[l];
                let before = adj.len();

                // rebuild instead of retain() that closes over &self.nodes
                let mut new_adj = Vec::with_capacity(adj.len());
                for &x in adj.iter() {
                    if x < nlen && x != nid && !deleted[x] {
                        new_adj.push(x);
                    }
                }
                new_adj.sort_unstable();
                new_adj.dedup();
                edges_dropped += before.saturating_sub(new_adj.len());
                *adj = new_adj;
            }
        }

        // Rebuild level registry
        self.max_level = self.nodes
            .iter()
            .map(|n| n.links.len().saturating_sub(1))
            .max()
            .unwrap_or(0);

        self.levels.clear();
        self.levels.resize(self.max_level + 1, Vec::new());
        for (nid, n) in self.nodes.iter().enumerate() {
            if !n.is_deleted() {
                let top = n.links.len().saturating_sub(1);
                self.levels[top].push(nid);
            }
        }

        // Entry and ext-id map
        self.entry = self.pick_entry();
        self.by_ext.clear();
        for (nid, n) in self.nodes.iter().enumerate() {
            if !n.is_deleted() {
                self.by_ext.insert(n.ext_id, nid);
            }
        }

        // Counters
        self.active = self.nodes.iter().filter(|n| !n.is_deleted()).count();
        self.total_bytes = self.nodes.iter_mut().map(|n| n.recompute_bytes()).sum();

        (edges_dropped, nodes_fixed)
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------
// Maintenance / Management API
// ----------------------------

impl Graph {
    #[inline]
    fn is_valid_nid(&self, nid: NodeId) -> bool {
        nid < self.nodes.len() && !self.nodes[nid].is_deleted()
    }

    /// Find a non-deleted node from the highest non-empty level down.
    fn pick_entry(&self) -> Option<NodeId> {
        for lvl in (0..self.levels.len()).rev() {
            for &nid in &self.levels[lvl] {
                if self.is_valid_nid(nid) { return Some(nid); }
            }
        }
        None
    }
}

impl Graph {
    /// Idempotent delete by external id. Returns true if something was removed.
    pub fn delete(&mut self, ext_id: u64) -> bool {
        let Some(nid) = self.by_ext.remove(&ext_id) else { return false; };
        if nid >= self.nodes.len() { return false; }
        if self.nodes[nid].is_deleted() { return false; }

        // We'll accumulate the net byte delta for neighbors + node and then
        // apply it once to self.total_bytes (can be negative).
        let mut bytes_delta: isize = 0;

        // Capture node's bytes before any modifications.
        let node_bytes_before = { self.nodes[nid].recompute_bytes() };

        // Unlink from neighbors at each existing level, updating their byte footprints.
        let levels = { self.nodes[nid].links.len() };
        for l in 0..levels {
            // Remove nid from every neighbor's adjacency list on layer l.
            // Take this level's neighbors from the node, then drop the borrow of node.
            let neigh: Vec<NodeId> = {
                let node = &mut self.nodes[nid];
                std::mem::take(&mut node.links[l])
            };
            for nb in neigh {
                if nb >= self.nodes.len() { continue; }
                if self.nodes[nb].is_deleted() { continue; }
                if l >= self.nodes[nb].links.len() { continue; }
                // neighbor bytes before
                let nb_bytes_before = { self.nodes[nb].recompute_bytes() };
                {
                    let adj = &mut self.nodes[nb].links[l];
                    if let Some(pos) = adj.iter().position(|&x| x == nid) {
                        adj.swap_remove(pos);
                    }
                }
                // neighbor bytes after
                let nb_bytes_after = { self.nodes[nb].recompute_bytes() };
                bytes_delta += (nb_bytes_after as isize) - (nb_bytes_before as isize);
            }
        }

        // After removing all links from this node, recalc its bytes (links now empty).
        let node_bytes_after_links = { self.nodes[nid].recompute_bytes() };
        bytes_delta += (node_bytes_after_links as isize) - (node_bytes_before as isize);

        // Clear vector memory (shrink) and mark deleted), and add the byte delta.
        bytes_delta += {
            let before = self.nodes[nid].recompute_bytes();
            {
                let node = &mut self.nodes[nid];
                node.vec.clear();         // release vector contents
                node.vec.shrink_to_fit(); // return capacity
                node.deleted
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
            let after = self.nodes[nid].recompute_bytes();
            (after as isize) - (before as isize)
        };

        self.active = self.active.saturating_sub(1);

        // Remove from level registry lists where present.
        for l in 0..self.levels.len() {
            if let Some(pos) = self.levels[l].iter().position(|&x| x == nid) {
                self.levels[l].swap_remove(pos);
            }
        }

        // Maintain entry: if we deleted the entry, pick a fallback if any.
        if self.entry == Some(nid) {
            self.entry = self.levels.iter().rev().find_map(|lvl| lvl.first().copied());
        }

        // Apply accumulated delta to total_bytes.
        if bytes_delta >= 0 {
            self.total_bytes = self.total_bytes.saturating_add(bytes_delta as usize);
        } else {
            self.total_bytes = self.total_bytes.saturating_sub((-bytes_delta) as usize);
        }
        true
    }

    /// Touch multiple external ids with the same timestamp (from search hits).
    pub fn touch_many(&self, ids: &[u64], now_unix: u64) {
        for &eid in ids {
            if let Some(&nid) = self.by_ext.get(&eid) {
                if nid < self.nodes.len() {
                    let node = &self.nodes[nid];
                    if !node.is_deleted() {
                        node.touch(now_unix);
                    }
                }
            }
        }
    }

    /// Basic stats for eviction/monitoring.
    #[inline]
    pub fn stats(&self) -> (usize, usize) {
        (self.active, self.total_bytes)
    }

    /// TTL sweep: evict nodes whose last_hit is older than `ttl_secs`.
    pub fn evict_ttl(&mut self, ttl_secs: u64, now_unix: u64) -> (usize, usize) {
        let mut evicted = 0usize;
        for nid in 0..self.nodes.len() {
            if self.nodes[nid].is_deleted() { continue; }
            let ts = self.nodes[nid].last_hit.load(std::sync::atomic::Ordering::Relaxed);
            if now_unix.saturating_sub(ts) > ttl_secs {
                let ext = self.nodes[nid].ext_id;
                if self.delete(ext) { evicted += 1; }
            }
        }
        self.repair_after_mass_deletes();
        (evicted, 0)
    }

    /// LRU eviction until caps are satisfied.
    pub fn evict_lru_until(
        &mut self,
        max_vecs: Option<usize>,
        max_bytes: Option<usize>,
        _now_unix: u64,
    ) -> (usize, usize) {
        let need = |active: usize, bytes: usize| {
            if let Some(mv) = max_vecs { if active > mv { return true; } }
            if let Some(mb) = max_bytes { if bytes > mb { return true; } }
            false
        };

        let (mut active, mut bytes) = self.stats();
        if !need(active, bytes) { return (0, 0); }

        let mut heap: std::collections::BinaryHeap<std::cmp::Reverse<(u64, NodeId)>> = std::collections::BinaryHeap::new();
        for (nid, n) in self.nodes.iter().enumerate() {
            if !n.is_deleted() {
                let ts = n.last_hit.load(std::sync::atomic::Ordering::Relaxed);
                heap.push(std::cmp::Reverse((ts, nid)));
            }
        }

        let mut evicted = 0usize;
        while let Some(std::cmp::Reverse((_ts, nid))) = heap.pop() {
            if !need(active, bytes) { break; }
            if nid >= self.nodes.len() { continue; }
            let ext = self.nodes[nid].ext_id;
            if self.delete(ext) {
                (active, bytes) = self.stats();
                evicted += 1;
            }
        }
        self.repair_after_mass_deletes();
        (evicted, 0)
    }
}

impl Graph {
    fn repair_after_mass_deletes(&mut self) {
        // Drop empty tails in levels and fix max_level
        while self.max_level > 0 && self.levels[self.max_level].is_empty() {
            self.max_level -= 1;
            self.levels.pop();
        }
        // If entry is gone or invalid, pick a new one
        if self.entry.map_or(true, |e| !self.is_valid_nid(e)) {
            self.entry = self.pick_entry();
        }
    }
}
