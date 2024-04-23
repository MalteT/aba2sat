use std::collections::{BTreeMap, HashSet};

use graph_cycles::Cycles;
use iter_tools::Itertools;

use crate::{
    aba::Num,
    clauses::Clause,
    literal::{
        lits::{LoopHelper, TheoryRuleBodyActive},
        IntoLiteral,
    },
};

use super::{theory::theory_helper, Aba, Context};

#[derive(Debug, Clone, PartialEq, Eq)]
struct r#Loop {
    heads: HashSet<Num>,
    support: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedAba {
    aba: Aba,
    loops: Vec<r#Loop>,
}

impl PreparedAba {
    /// Create a new [`PreparedAba`] from a raw [`Aba`]
    pub fn new(mut aba: Aba, max_loops: Option<usize>) -> Self {
        trim_unreachable_rules(&mut aba);
        let loops = calculate_loops_and_their_support(&aba, max_loops);
        PreparedAba { aba, loops }
    }
    /// Translate the ABA into base rules / definitions for SAT solving
    pub fn derive_clauses<Ctx: Context>(&self) -> impl Iterator<Item = Clause> + '_ {
        theory_helper::<Ctx>(self)
            .chain(self.derive_loop_breaker::<Ctx>())
            .chain(self.derive_rule_helper::<Ctx>())
    }

    /// Derive [`Clause`]s to ground the found loops
    ///
    /// Given the loop based on these rules
    /// ```text
    /// p <- q
    /// q <- p
    /// q <- a
    /// p <- b
    /// ```
    /// the breaker will derive the formulas
    /// ```text
    /// p => a v b
    /// q => a v b
    /// ```
    /// or, in the more general case for Loop `L` and incoming rules `Ri = {ri1, ..., rin}`, where all elements of the body of a rule are outside of the loop, we have for all elements `l in L` with id `i`:
    /// ```text
    /// l => and(body(ri1)) or ... or and(body(rln))
    /// ```
    /// where body(h <- B) = B and and({a1, ..., ax}) = a1 and ... and ax.
    ///
    /// This will lead to an exponential explosion when converted to CNF naively,
    /// since the formulas are mostly DNF. We use Tseitin's transformation to prevent this:
    /// ```text
    ///    LH_i <=> RBA_1 or ... or RBA_n
    /// ⋄  (LH_i or -RBA_1) and ... and (LH_i or -RBA_n) and (-LH_i or RBA_1 or ... or RBA_n)
    ///
    ///    l => LH_i
    /// ⋄  -l or LH_i
    /// ```
    /// This will result in `|L| + 1` new clauses per loop.
    fn derive_loop_breaker<Ctx: Context>(&self) -> impl Iterator<Item = Clause> + '_ {
        // Iterate over all loops
        self.loops.iter().enumerate().flat_map(|(loop_id, r#loop)| {
            // -LH_i or RBA_1 or ... or RBA_n
            let last_clause = r#loop
                .support
                .iter()
                .map(|el| TheoryRuleBodyActive::from(*el).pos())
                .chain(std::iter::once(LoopHelper::from(loop_id).neg()))
                .collect();
            // -l or LH_i
            let head_clauses = r#loop.heads.iter().map(move |head| {
                Clause::from(vec![
                    LoopHelper::from(loop_id).pos(),
                    Ctx::Base::from(*head).neg(),
                ])
            });
            // LH_i or -RBA_x
            let tuple_clauses = r#loop.support.iter().map(move |rule_id| {
                Clause::from(vec![
                    TheoryRuleBodyActive::from(*rule_id).neg(),
                    LoopHelper::from(loop_id).pos(),
                ])
            });
            tuple_clauses.chain([last_clause]).chain(head_clauses)
        })
    }

    /// Derive helper for every rule
    ///
    /// This simplifies some thinks massively and is used by the loop breaker
    /// and prevents exponential explosion for rules with the same head.
    ///
    /// For a rule `h <- b_1, ..., b_n with index i in R`, create a helper
    /// ```text
    ///    RBA_i <=> b_1 and ... and b_n
    /// ⋄  (-RBA_i or b_1) and ... and (-RBA_i or b_n) and (RBA_i or -b_1 or ... or -b_n)
    /// ```
    /// we will use the `TheoryRuleActive` for `x_R`.
    fn derive_rule_helper<Ctx: Context>(&self) -> impl Iterator<Item = Clause> + '_ {
        self.rules
            .iter()
            .enumerate()
            .flat_map(|(rule_id, (_head, body))| {
                if body.is_empty() {
                    vec![Clause::from(vec![Ctx::Rule::from(rule_id).neg()])]
                } else {
                    let last_clause = body
                        .iter()
                        .map(|el| Ctx::Base::from(*el).neg())
                        .chain(std::iter::once(Ctx::Rule::from(rule_id).pos()))
                        .collect();
                    body.iter()
                        .map(move |el| {
                            Clause::from(vec![
                                Ctx::Rule::from(rule_id).neg(),
                                Ctx::Base::from(*el).pos(),
                            ])
                        })
                        .chain([last_clause])
                        .collect()
                }
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

fn calculate_loops_and_their_support(aba: &Aba, max_loops: Option<usize>) -> Vec<r#Loop> {
    let mut graph = petgraph::graph::DiGraph::<Num, ()>::new();
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
    #[cfg(debug_assertions)]
    {
        use std::{fs::File, io::Write};
        let mut file = File::create("./graph.gv").unwrap();
        let dot = petgraph::dot::Dot::with_config(&graph, &[petgraph::dot::Config::EdgeNoLabel]);
        write!(file, "{dot:?}").unwrap();
    }
    let mut loops = vec![];
    const LOOP_SIZE_IN_MULT_UNIVERSE_SIZE: f32 = 1.0;
    let max_loops = if let Some(max) = max_loops {
        max
    } else {
        (universe.len() as f32 * LOOP_SIZE_IN_MULT_UNIVERSE_SIZE) as usize
    };
    let mut output_printed = false;
    graph.visit_cycles(|graph, cycle| {
        let heads = cycle.iter().map(|idx| graph[*idx]).collect::<HashSet<_>>();
        let loop_rules = aba
            .rules
            .iter()
            .enumerate()
            .filter(|(_rule_id, (head, _body))| heads.contains(head));
        // Relevant rules are those that contain only elements from outside the loop
        // All other rules cannot influence the value of the loop
        let support = loop_rules
            .filter(|(_rule_id, (_head, body))| body.is_disjoint(&heads))
            .map(|(rule_id, _)| rule_id)
            .collect();
        loops.push(r#Loop { heads, support });
        if loops.len() >= max_loops {
            if ! output_printed {
                eprintln!("Too... many... cycles... Aborting cycle detection with {} cycles. Solver? You're on your own now", loops.len());
                output_printed = true;
            }
            std::ops::ControlFlow::Break(())
        } else {
            std::ops::ControlFlow::Continue(())
        }
    });
    loops
}

impl std::ops::Deref for PreparedAba {
    type Target = Aba;

    fn deref(&self) -> &Self::Target {
        &self.aba
    }
}
