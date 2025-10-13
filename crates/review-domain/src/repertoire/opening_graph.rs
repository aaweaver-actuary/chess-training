use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use crate::RepertoireMove;

#[derive(Clone, Debug, Default)]
pub struct OpeningGraph {
    edges: Vec<RepertoireMove>,
    by_parent: HashMap<u64, Vec<usize>>,
    by_child: HashMap<u64, Vec<usize>>,
    roots: Vec<u64>,
    positions: HashSet<u64>,
}

impl OpeningGraph {
    #[must_use]
    pub fn from_moves<I>(moves: I) -> Self
    where
        I: IntoIterator<Item = RepertoireMove>,
    {
        let mut edges = Vec::new();
        let mut by_parent: HashMap<u64, Vec<usize>> = HashMap::new();
        let mut by_child: HashMap<u64, Vec<usize>> = HashMap::new();
        let mut positions: HashSet<u64> = HashSet::new();

        for move_entry in moves {
            positions.insert(move_entry.parent_id);
            positions.insert(move_entry.child_id);

            let edge_index = edges.len();
            by_parent
                .entry(move_entry.parent_id)
                .or_default()
                .push(edge_index);
            by_child
                .entry(move_entry.child_id)
                .or_default()
                .push(edge_index);

            edges.push(move_entry);
        }

        let mut roots = BTreeSet::new();
        for position in &positions {
            if !by_child.contains_key(position) {
                roots.insert(*position);
            }
        }

        Self {
            edges,
            by_parent,
            by_child,
            roots: roots.into_iter().collect(),
            positions,
        }
    }

    pub fn children(&self, position_id: u64) -> impl Iterator<Item = &RepertoireMove> {
        self.by_parent
            .get(&position_id)
            .into_iter()
            .flatten()
            .map(|&idx| &self.edges[idx])
    }

    pub fn parents(&self, position_id: u64) -> impl Iterator<Item = &RepertoireMove> {
        self.by_child
            .get(&position_id)
            .into_iter()
            .flatten()
            .map(|&idx| &self.edges[idx])
    }

    #[must_use]
    pub fn path_to(&self, position_id: u64) -> Option<Vec<&RepertoireMove>> {
        if !self.positions.contains(&position_id) {
            return None;
        }

        if self.roots.contains(&position_id) {
            return Some(Vec::new());
        }

        let mut visited = HashSet::new();
        let mut queue: VecDeque<(u64, Vec<usize>)> = VecDeque::new();

        for &root in &self.roots {
            queue.push_back((root, Vec::new()));
        }

        while let Some((current, path_indices)) = queue.pop_front() {
            if current == position_id {
                let path = path_indices
                    .into_iter()
                    .map(|idx| &self.edges[idx])
                    .collect();
                return Some(path);
            }

            if !visited.insert(current) {
                continue;
            }

            if let Some(children) = self.by_parent.get(&current) {
                for &edge_idx in children {
                    let mut next_path = path_indices.clone();
                    next_path.push(edge_idx);
                    let child = self.edges[edge_idx].child_id;
                    queue.push_back((child, next_path));
                }
            }
        }

        None
    }

    #[must_use]
    pub fn roots(&self) -> &[u64] {
        &self.roots
    }
}
