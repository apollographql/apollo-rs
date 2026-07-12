use crate::name::Name;
use indexmap::IndexSet;
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::Bfs;
use petgraph::visit::Reversed;
use std::collections::HashMap;

/// Directed graph of `implements` edges (`X implements Y` ⇒ edge `X →
/// Y`). Holds both interface→interface and object→interface edges.
#[derive(Debug, Clone, Default)]
pub(crate) struct ImplementsGraph {
    graph: DiGraph<Name, ()>,
    by_name: HashMap<Name, NodeIndex>,
}

impl ImplementsGraph {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Record `from implements to`. Idempotent.
    pub(crate) fn add_edge(&mut self, from: &Name, to: &Name) {
        let f = self.node_for(from);
        let t = self.node_for(to);
        if !self.graph.contains_edge(f, t) {
            self.graph.add_edge(f, t, ());
        }
    }

    /// `start` plus every name reachable from it.
    pub(crate) fn closure(&self, start: &Name) -> IndexSet<Name> {
        let Some(&root) = self.by_name.get(start) else {
            return IndexSet::new();
        };
        let mut bfs = Bfs::new(&self.graph, root);
        let mut out = IndexSet::new();
        while let Some(idx) = bfs.next(&self.graph) {
            out.insert(self.graph[idx].clone());
        }
        out
    }

    /// `name`'s directly-picked parents (one edge out), not the closure.
    pub(crate) fn direct_parents(&self, name: &Name) -> IndexSet<Name> {
        let Some(&idx) = self.by_name.get(name) else {
            return IndexSet::new();
        };
        self.graph
            .neighbors(idx)
            .map(|n| self.graph[n].clone())
            .collect()
    }

    /// Names ordered with every parent before its children, so a pass
    /// reading only direct parents sees them already reconciled. Falls
    /// back to insertion order on cycle (which shouldn't happen — picks
    /// reject candidates whose closure would loop back).
    pub(crate) fn topo_order_parents_first(&self) -> Vec<Name> {
        match toposort(&Reversed(&self.graph), None) {
            Ok(order) => order
                .into_iter()
                .map(|idx| self.graph[idx].clone())
                .collect(),
            Err(_) => self.by_name.keys().cloned().collect(),
        }
    }

    pub(crate) fn node_for(&mut self, name: &Name) -> NodeIndex {
        if let Some(&idx) = self.by_name.get(name) {
            return idx;
        }
        let idx = self.graph.add_node(name.clone());
        self.by_name.insert(name.clone(), idx);
        idx
    }
}
