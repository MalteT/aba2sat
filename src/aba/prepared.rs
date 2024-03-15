use std::collections::HashSet;

use iter_tools::Itertools;

use crate::{aba::Num, clauses::Clause, graph::Graph, literal::TheoryAtom};

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
            let loop_enforcement_clauses =
                head_list
                    .into_iter()
                    .tuple_windows()
                    .flat_map(|(first, second)| {
                        [
                            Clause::from(vec![I::new(*first).pos(), I::new(*second).pos()]),
                            Clause::from(vec![I::new(*first).neg(), I::new(*second).neg()]),
                        ]
                    });
            let head_sample = *r#loop.heads.iter().next().unwrap();
            let body_rules = r#loop.support.iter().map(|(_head, body)| body);
            let clauses = body_rules.multi_cartesian_product().map(move |product| {
                product
                    .into_iter()
                    .map(|elem| I::new(*elem).pos())
                    .chain(std::iter::once(I::new(head_sample).neg()))
                    .collect()
            });
            loop_enforcement_clauses.chain(clauses)
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
    // Construct the graph containing all elements of the universe
    // with edges based on the aba's rules
    let graph = aba.rules.iter().fold(Graph::new(), |graph, (head, body)| {
        body.iter().fold(graph, |mut graph, elem| {
            graph.add_edge(*elem, *head);
            graph
        })
    });
    // Use a linear time algorithm to calculate the strongly connected
    // components of the derived graph
    let scc = graph.tarjan_scc();
    // Loops are strongly connected components that have more than one element
    // These are just the largest loops. There may be smaller loops inside loops
    // but it should suffice to prevent these loops
    let loops = scc.into_iter().filter(|component| component.len() > 1);
    // Iterate over all loops and apply the fixing-logic
    loops
        .map(|heads| {
            let heads: HashSet<_> = heads.into_iter().map(|head| head as Num).collect();
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
            r#Loop { heads, support }
        })
        .collect()
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
