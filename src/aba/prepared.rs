use std::{
    collections::{BTreeMap, HashSet},
    fs::File,
    io::Write,
};

use graph_cycles::Cycles;
use iter_tools::Itertools;
use petgraph::{
    dot::{Config, Dot},
    graph::DiGraph,
};

use crate::{aba::Num, clauses::Clause, literal::TheoryAtom};

use super::{theory::theory_helper, Aba, RuleList};

#[derive(Debug, Clone, PartialEq, Eq)]
struct r#Loop {
    heads: HashSet<Num>,
    support: RuleList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedAba {
    aba: Aba,
    loops: Vec<r#Loop>,
}

impl PreparedAba {
    /// Translate the ABA into base rules / definitions for SAT solving
    pub fn derive_clauses<I: TheoryAtom>(&self) -> impl Iterator<Item = Clause> + '_ {
        theory_helper::<I>(self).chain(self.derive_loop_breaker::<I>())
    }

    fn derive_loop_breaker<I: TheoryAtom>(&self) -> impl Iterator<Item = Clause> + '_ {
        self.loops.iter().flat_map(|r#loop| {
            let mut head_list: Vec<_> = r#loop.heads.iter().collect();
            head_list.push(head_list[0]);
            let body_rules = r#loop.support.iter().map(|(_head, body)| body);
            let clauses = body_rules
                .multi_cartesian_product()
                .flat_map(move |product| {
                    r#loop.heads.iter().map(move |head| {
                        product
                            .iter()
                            .map(|elem| I::new(**elem).pos())
                            .chain(std::iter::once(I::new(*head).neg()))
                            .collect()
                    })
                });
            clauses
        })
    }
}

/// Filtered list of rules
///
/// Iterates over all rules, marking reachable elements until
/// no additional rule can be applied. Then removes every
/// rule that contains any unreachable atom and returns the rest
fn trim_unreachable_rules(aba: &mut Aba) {
    // Begin with all assumptions marked as reachable
    let mut reachable: HashSet<_> = aba.assumptions().cloned().collect();
    // Calculate all reachable elements
    loop {
        let mut marked_any = false;
        for (head, body) in &aba.rules {
            if reachable.contains(head) {
                continue;
            }
            if body.iter().all(|atom| reachable.contains(atom)) {
                marked_any = true;
                reachable.insert(*head);
            }
        }
        if !marked_any {
            break;
        }
    }
    // Remove all rules that contain any unreachable atom
    aba.rules.retain(|(head, body)| {
        // Both the head and all elements from the body must be reachable
        reachable.contains(head) && body.iter().all(|atom| reachable.contains(atom))
    });
}

fn calculate_loops_and_their_support(aba: &Aba) -> Vec<r#Loop> {
    let mut graph = DiGraph::<Num, ()>::new();
    let universe = aba
        .universe()
        .unique()
        .scan(&mut graph, |graph, element| {
            let idx = graph.add_node(*element);
            Some((*element, idx))
        })
        .collect::<BTreeMap<_, _>>();
    aba.rules
        .iter()
        .flat_map(|(head, body)| body.iter().map(|body_element| (*body_element, *head)))
        .for_each(|(from, to)| {
            let from = universe.get(&from).unwrap();
            let to = universe.get(&to).unwrap();
            graph.update_edge(*from, *to, ());
        });
    let mut file = File::create("./graph.gv").unwrap();
    let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel]);
    write!(file, "{dot:?}").unwrap();
    // TODO: Write on debug, simplify for production
    let mut loops = vec![];
    graph.visit_cycles(|graph, cycle| {
        let heads = cycle.iter().map(|idx| graph[*idx]).collect::<HashSet<_>>();
        let loop_rules = aba
            .rules
            .iter()
            .filter(|(head, _body)| heads.contains(head));
        // Relevant rules are those that contain only elements from outside the loop
        // All other rules cannot influence the value of the loop
        let support = loop_rules
            .filter(|(_head, body)| body.is_disjoint(&heads))
            .cloned()
            .collect();
        loops.push(r#Loop { heads, support });
        if loops.len() >= universe.len() {
            if loops.len() == universe.len() {
                eprintln!("Too... many... cycles... Aborting cycle detection. Solver? You're on your own now");
            }
            std::ops::ControlFlow::Break(())
        } else {
            std::ops::ControlFlow::Continue(())
        }
    });
    loops
}

impl From<Aba> for PreparedAba {
    fn from(mut aba: Aba) -> Self {
        trim_unreachable_rules(&mut aba);
        let loops = calculate_loops_and_their_support(&aba);
        PreparedAba { aba, loops }
    }
}

impl std::ops::Deref for PreparedAba {
    type Target = Aba;

    fn deref(&self) -> &Self::Target {
        &self.aba
    }
}
