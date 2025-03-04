use std::collections::{BTreeMap, BTreeSet, HashSet};

use bit_set::BitSet;

use crate::STOP_LOOP_COUNTING;

use super::{Aba, Num, RuleList};

pub struct Loops {
    rem_loops: Option<usize>,
    sccs: Vec<Graph>,
    rem: Vec<BitSet<usize>>,
}

impl Loops {
    pub fn of(aba: &'_ Aba, max_loops: Option<usize>) -> Self {
        // Set the global stopper to false;
        STOP_LOOP_COUNTING.store(false, std::sync::atomic::Ordering::Relaxed);
        Self {
            rem_loops: max_loops,
            sccs: Graph::compute_sccs(aba).collect(),
            rem: vec![],
        }
    }
}

/// A single loop within our [`Aba`]
///
/// Represented as a set of rule heads.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Loop {
    pub heads: BTreeSet<Num>,
}

/// Graph representation of our [`Aba`]
///
/// This is guaranteed to be a single strongly-connected component
/// with at least two nodes and thus at least two edges.
struct Graph {
    /// The edges of our graph
    ///
    /// An edge (s, t) exists if there is a rule t <- B with s in B.
    next: BTreeMap<Num, BTreeSet<Num>>,
}
impl Graph {
    fn compute_sccs(aba: &Aba) -> impl Iterator<Item = Self> + '_ {
        compute_sccs(&aba.rules)
            .into_iter()
            .filter(|scc| scc.len() >= 2)
            .map(|scc| {
                aba.rules
                    .iter()
                    .flat_map(|(head, body)| body.iter().map(|body_element| (*body_element, *head)))
                    .filter(|(from, to)| scc.contains(from) && scc.contains(to))
                    .fold(
                        BTreeMap::<Num, BTreeSet<Num>>::new(),
                        |mut next, (from, to)| {
                            next.entry(from).or_default().insert(to);
                            next
                        },
                    )
            })
            .map(|next| Graph { next })
    }

    fn compute_loops(&self, max_loops: Option<usize>) -> Vec<BitSet<usize>> {
        let max_loops = max_loops.unwrap_or(usize::MAX);
        #[derive(Debug)]
        struct Frame {
            node: Num,
            open: BTreeSet<Num>,
        }
        let mut stack: Vec<Frame> = vec![{
            let node = self.random_node();
            Frame {
                open: self.next.get(&node).unwrap().clone(),
                node,
            }
        }];
        let mut loops: Vec<BitSet<usize>> = vec![];
        // Begin!
        loop {
            if stack.is_empty()
                || loops.len() >= max_loops
                || STOP_LOOP_COUNTING.load(std::sync::atomic::Ordering::Relaxed)
            {
                break;
            }
            // Push to the stack!
            {
                // Get the last frame, this is safe, we've just checked for non-emptiness
                let last_frame = stack.last().unwrap();
                let next = last_frame.open.iter().next().cloned();
                match next {
                    Some(next) => {
                        // remove next from the set of open edges
                        let last = stack.len() - 1;
                        stack[last].open.remove(&next);
                        // push new frame
                        stack.push(Frame {
                            open: self.next.get(&next).cloned().unwrap_or_default(),
                            node: next,
                        });
                    }
                    None => {
                        // There are no next nodes to try, pop that frame and start over
                        stack.pop();
                        continue;
                    }
                }
            }
            // At this point the stack cannot be empty
            let frame = stack.last().unwrap();
            // Check whether this is a loop
            let other_index = stack[0..stack.len() - 1]
                .iter()
                .enumerate()
                .find(|(_, f)| f.node == frame.node);
            match other_index {
                Some((idx, _)) => {
                    // We have found a loop!
                    let new_loop = stack[idx + 1..].iter().map(|f| f.node as usize).collect();
                    if !loops.iter().any(|l| *l == new_loop) {
                        // It's a novel loop!
                        loops.push(new_loop);
                    }
                    // drop the current frame
                    stack.pop();
                }
                None => {
                    // Not a loop yet
                    // TODO: continue
                }
            }
        }
        // calculate joined loops
        let mut left_idx = 0;
        // the left index will walk once through all loops, including new ones
        'outer: while left_idx < loops.len() && loops.len() < max_loops {
            // the right index will walk through the entire list for every left index
            for right_idx in 0..loops.len() {
                if left_idx == right_idx {
                    continue;
                }
                let left = &loops[left_idx];
                let right = &loops[right_idx];
                // left \cap right
                let mut new = left.clone();
                new.intersect_with(right);
                // check that left and right really intersect AND the intersection differs from both sets
                if !new.is_empty() && new != *right && new != *left {
                    // make new the union of left and right
                    new.union_with(left);
                    new.union_with(right);
                    if !loops.iter().any(|l| new == *l) {
                        loops.push(new)
                    }
                    if loops.len() >= max_loops {
                        break 'outer;
                    }
                }
            }
            left_idx += 1;
        }
        loops
    }

    /// A random node from this graph
    fn random_node(&self) -> Num {
        // Safe, the graph is guaranteed to not be empty!
        *self.next.keys().next().unwrap()
    }
}

fn compute_sccs(rules: &RuleList) -> Vec<BTreeSet<Num>> {
    // We're only interested in atoms that can be reached via a rule.
    // Since every head may have multiple rules, we join their bodies here.
    // Instead of using the direction `body` -> `head` as our `edge`, we
    // invert that, since sccs are unchanged by a flip of the edge and we
    // already have all the info on hand
    let succ = rules.iter().fold(
        BTreeMap::<Num, HashSet<Num>>::new(),
        |mut map, (head, body)| {
            map.entry(*head).or_default().extend(body);
            map
        },
    );
    let mut sccs = vec![];
    // current tarjan index
    let mut index = 0;
    // The stack (our SCC in the making)
    let mut stack: Vec<Num> = vec![];
    /// The tuple (index, lowlink, onStack)
    #[derive(Debug)]
    struct Info {
        index: usize,
        lowlink: usize,
        on_stack: bool,
    }
    // information about all elements of our "graph"
    let mut info: BTreeMap<Num, Info> = BTreeMap::new();
    // Tarjans strongconnect function
    fn strong_connect(
        sccs: &mut Vec<BTreeSet<Num>>,
        succ: &BTreeMap<Num, HashSet<Num>>,
        info: &mut BTreeMap<Num, Info>,
        stack: &mut Vec<Num>,
        index: &mut usize,
        node: Num,
    ) {
        // Set the node's index and lowlink and on_stack
        info.insert(
            node,
            Info {
                index: *index,
                lowlink: *index,
                on_stack: true,
            },
        );
        *index += 1;
        // Push the node onto the stack
        stack.push(node);
        // an extra check is necessary here, as some atoms may not be
        // head to a rule, like assumptions
        let successors = match succ.get(&node) {
            Some(successor) => successor,
            None => {
                return;
            }
        };
        // Iterate over successor nodes
        for successor in successors {
            match info.get(successor) {
                // Not yet visited
                None => {
                    strong_connect(sccs, succ, info, stack, index, *successor);
                    let successor_lowlink = info.get(successor).unwrap().lowlink;
                    let node_info = info.get_mut(&node).unwrap();
                    node_info.lowlink = node_info.lowlink.min(successor_lowlink);
                }
                // Visited and in the same SCC
                Some(i) if i.on_stack => {
                    let successor_index = info.get(successor).unwrap().index;
                    let node_info = info.get_mut(&node).unwrap();
                    node_info.lowlink = node_info.lowlink.min(successor_index);
                }
                // Already part of another SCC
                _ => {}
            }
        }
        let node_info = info.get(&node).unwrap();
        // This is the root node of the SCC
        if node_info.lowlink == node_info.index {
            // create a new SCC from the stack
            let mut scc = BTreeSet::new();
            loop {
                // Remove the SCC from the stack
                let popped = stack.pop().unwrap();
                scc.insert(popped);
                let popped_info = info.get_mut(&popped).unwrap();
                popped_info.on_stack = false;
                if popped == node {
                    break;
                }
            }
            sccs.push(scc)
        }
    }
    // iterate over all elements and their successors
    for node in succ.keys() {
        if !info.contains_key(node) {
            strong_connect(&mut sccs, &succ, &mut info, &mut stack, &mut index, *node)
        }
    }
    // Return the strongly connected components
    sccs
}

impl Iterator for Loops {
    type Item = Loop;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rem_loops == Some(0) {
            return None;
        }
        match self.rem.pop() {
            Some(heads) => {
                self.rem_loops = self.rem_loops.map(|rem| rem - 1);
                Some(Loop {
                    heads: heads.into_iter().map(|raw| raw as Num).collect(),
                })
            }
            None if self.sccs.is_empty() => None,
            None => {
                let next = self.sccs.pop().unwrap();
                self.rem = next.compute_loops(self.rem_loops);
                self.next()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::aba::debug::DebugAba;

    use super::*;

    #[test]
    fn no_loops() {
        let aba = Aba::default();
        let loops = Loops::of(&aba, None).count();
        assert_eq!(loops, 0);

        let mut loops = Loops::of(&aba, None);
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
    }

    #[test]
    fn a_single_loop() {
        let aba = DebugAba::default()
            .with_assumption('a', 'q')
            .with_rule('p', ['q'])
            .with_rule('q', ['p'])
            .with_rule('p', ['a']);
        let the_loop = aba
            .forward_set(['p', 'q'].into_iter().collect())
            .unwrap()
            .into_iter()
            .collect();
        let loops = Loops::of(aba.aba(), None).count();
        assert_eq!(loops, 1);

        let mut loops = Loops::of(&aba.aba(), None);
        let first = loops.next().unwrap();
        assert_eq!(first.heads, the_loop);
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
    }

    #[test]
    fn two_loops() {
        let aba = DebugAba::default()
            .with_assumption('a', 'q')
            .with_rule('p', ['a'])
            .with_rule('q', ['p'])
            .with_rule('p', ['q'])
            .with_rule('r', ['q'])
            .with_rule('p', ['r']);
        let first_loop = aba
            .forward_set(['p', 'q'].into_iter().collect())
            .unwrap()
            .into_iter()
            .collect();
        let second_loop = aba
            .forward_set(['p', 'q', 'r'].into_iter().collect())
            .unwrap()
            .into_iter()
            .collect();
        let loops = Loops::of(aba.aba(), None).count();
        assert_eq!(loops, 2);

        let mut loops = Loops::of(&aba.aba(), None);
        let next = loops.next().unwrap();
        assert!(next.heads == first_loop || next.heads == second_loop);
        let next = loops.next().unwrap();
        assert!(next.heads == first_loop || next.heads == second_loop);
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
    }

    #[test]
    fn three_loops() {
        let aba = DebugAba::default()
            .with_assumption('a', 'q')
            .with_rule('p', ['a'])
            .with_rule('q', ['p'])
            .with_rule('p', ['q'])
            .with_rule('r', ['q'])
            .with_rule('p', ['r'])
            .with_rule('r', ['p']);
        let expected = [
            aba.forward_set(['p', 'q'].into_iter().collect())
                .unwrap()
                .into_iter()
                .collect(),
            aba.forward_set(['p', 'q', 'r'].into_iter().collect())
                .unwrap()
                .into_iter()
                .collect(),
            aba.forward_set(['p', 'r'].into_iter().collect())
                .unwrap()
                .into_iter()
                .collect(),
        ];
        let mut loops = Loops::of(&aba.aba(), None);
        for _number in 0..expected.len() {
            let next = loops.next().unwrap();
            assert!(
                expected.contains(&next.heads),
                "Unexpected loop {:?}",
                next.heads
            );
        }
        // The iterator should be empty now
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
    }

    #[test]
    fn scc_test() {
        let aba = DebugAba::default()
            .with_rule('p', ['q'])
            .with_rule('q', ['p'])
            .with_rule('u', ['v'])
            .with_rule('v', ['w'])
            .with_rule('w', ['u']);
        let rules = &aba.aba().rules;
        let scc_indizes = compute_sccs(rules);
        let expected: Vec<BTreeSet<Num>> = vec![
            vec![1, 2].into_iter().collect(),
            vec![3, 4, 5].into_iter().collect(),
        ];
        assert_eq!(expected, scc_indizes);
    }
}
