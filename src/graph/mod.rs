//! [SOURCE](https://github.com/TheAlgorithms/Rust)
//!
//! MIT License
//!
//! Copyright (c) 2019 The Algorithms
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::aba::Num;

#[derive(Debug)]
pub struct Graph {
    adj_list: BTreeMap<Num, Vec<Num>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            adj_list: BTreeMap::new(),
        }
    }

    #[cfg(test)]
    pub fn add_vertex(&mut self, vertex: Num) {
        self.adj_list.entry(vertex).or_default();
    }

    pub fn add_edge(&mut self, from: Num, to: Num) {
        self.adj_list.entry(from).or_default().push(to);
    }

    pub fn tarjan_scc(&self) -> Vec<HashSet<Num>> {
        struct TarjanState {
            index: i32,
            stack: Vec<Num>,
            on_stack: BTreeSet<Num>,
            index_of: BTreeMap<Num, i32>,
            lowlink_of: BTreeMap<Num, i32>,
            components: Vec<HashSet<Num>>,
        }

        let mut state = TarjanState {
            index: 0,
            stack: Vec::new(),
            on_stack: Default::default(),
            index_of: Default::default(),
            lowlink_of: Default::default(),
            components: Vec::new(),
        };

        fn strong_connect(v: Num, graph: &Graph, state: &mut TarjanState) {
            state.index_of.insert(v, state.index);
            state.lowlink_of.insert(v, state.index);
            state.index += 1;
            state.stack.push(v);
            state.on_stack.insert(v);

            for &w in graph.adj_list.get(&v).unwrap_or(&vec![]) {
                if !state.index_of.contains_key(&w) {
                    strong_connect(w, graph, state);
                    let curr = state.lowlink_of.get(&v).cloned();
                    let other = state.lowlink_of.get(&w).cloned();
                    match (curr, other) {
                        (Some(curr), Some(other)) => {
                            state.lowlink_of.insert(v, curr.min(other));
                        }
                        (Some(first), None) | (None, Some(first)) => {
                            state.lowlink_of.insert(v, first);
                        }
                        (None, None) => {
                            state.lowlink_of.remove(&v);
                        }
                    }
                } else if state.on_stack.contains(&w) {
                    let curr = state.lowlink_of.get(&v).cloned();
                    let other = state.index_of.get(&w).cloned();
                    match (curr, other) {
                        (Some(curr), Some(other)) => {
                            state.lowlink_of.insert(v, curr.min(other));
                        }
                        (Some(first), None) | (None, Some(first)) => {
                            state.lowlink_of.insert(v, first);
                        }
                        (None, None) => {
                            state.lowlink_of.remove(&v);
                        }
                    }
                }
            }

            if state.lowlink_of.get(&v) == state.index_of.get(&v) {
                let mut component = HashSet::new();
                while let Some(w) = state.stack.pop() {
                    state.on_stack.remove(&w);
                    component.insert(w);
                    if w == v {
                        break;
                    }
                }
                state.components.push(component);
            }
        }

        for v in self.adj_list.keys() {
            if !state.index_of.contains_key(v) {
                strong_connect(*v, self, &mut state);
            }
        }

        state.components
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tarjan_scc() {
        // Test 1: A graph with multiple strongly connected components
        let n_vertices = 11;
        let edges = vec![
            (0, 1),
            (0, 3),
            (1, 2),
            (1, 4),
            (2, 0),
            (2, 6),
            (3, 2),
            (4, 5),
            (4, 6),
            (5, 6),
            (5, 7),
            (5, 8),
            (5, 9),
            (6, 4),
            (7, 9),
            (8, 9),
            (9, 8),
        ];
        let mut graph = Graph::new();
        (0..n_vertices).for_each(|v| graph.add_vertex(v));

        for &(u, v) in &edges {
            graph.add_edge(u, v);
        }

        let components = graph.tarjan_scc();
        assert_eq!(
            components,
            vec![
                set![8, 9],
                set![7],
                set![5, 4, 6],
                set![3, 2, 1, 0],
                set![10],
            ]
        );

        // Test 2: A graph with no edges
        let n_vertices = 5;
        let edges = vec![];
        let mut graph = Graph::new();
        (0..n_vertices).for_each(|v| graph.add_vertex(v));

        for &(u, v) in &edges {
            graph.add_edge(u, v);
        }

        let components = graph.tarjan_scc();

        // Each node is its own SCC
        assert_eq!(
            components,
            vec![set![0], set![1], set![2], set![3], set![4]]
        );

        // Test 3: A graph with single strongly connected component
        let n_vertices = 5;
        let edges = vec![(0, 1), (1, 2), (2, 3), (2, 4), (3, 0), (4, 2)];
        let mut graph = Graph::new();
        (0..n_vertices).for_each(|v| graph.add_vertex(v));

        for &(u, v) in &edges {
            graph.add_edge(u, v);
        }

        let components = graph.tarjan_scc();
        assert_eq!(components, vec![set![4, 3, 2, 1, 0]]);

        // Test 4: A graph with multiple strongly connected component
        let n_vertices = 7;
        let edges = vec![
            (0, 1),
            (1, 2),
            (2, 0),
            (1, 3),
            (1, 4),
            (1, 6),
            (3, 5),
            (4, 5),
        ];
        let mut graph = Graph::new();
        (0..n_vertices).for_each(|v| graph.add_vertex(v));

        for &(u, v) in &edges {
            graph.add_edge(u, v);
        }

        let components = graph.tarjan_scc();
        assert_eq!(
            components,
            vec![set![5], set![3], set![4], set![6], set![2, 1, 0],]
        );
    }
}
