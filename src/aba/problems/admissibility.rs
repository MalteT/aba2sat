use std::collections::HashSet;

use crate::{
    aba::{inference_helper, Aba},
    clauses::{Atom, Clause, ClauseList},
    literal::{Inference, IntoLiteral, Inverse, SetInference},
    mapper::Mapper,
};

use super::{LoopControl, MultishotProblem};

#[derive(Default, Debug)]
pub struct Admissibility<A: Atom> {
    found: Vec<HashSet<A>>,
}

impl<A: Atom> MultishotProblem<A> for Admissibility<A> {
    type Output = Vec<HashSet<A>>;

    fn additional_clauses(&self, aba: &Aba<A>, iteration: usize) -> ClauseList {
        match iteration {
            0 => {
                let mut clauses = vec![];
                // Create inference for the problem set
                inference_helper::<SetInference<_>, _>(&aba.rules).collect_into(&mut clauses);
                // Prevent the empty set
                let no_empty_set: Clause = aba
                    .inverses
                    .keys()
                    .map(|assumption| SetInference::new(assumption.clone()).pos())
                    .collect();
                clauses.push(no_empty_set);
                // Attack the inference of the aba, if an attack exists
                for assumption in aba.inverses.keys() {
                    for elem in aba.universe() {
                        clauses.push(Clause::from(vec![
                            SetInference::new(assumption.clone()).neg(),
                            Inverse::new(assumption.clone(), elem.clone()).neg(),
                            SetInference::new(elem.clone()).neg(),
                        ]));
                        clauses.push(Clause::from(vec![
                            SetInference::new(assumption.clone()).neg(),
                            Inverse::new(assumption.clone(), elem.clone()).neg(),
                            Inference::new(elem.clone()).neg(),
                        ]));
                        clauses.push(Clause::from(vec![
                            Inference::new(assumption.clone()).neg(),
                            Inverse::new(assumption.clone(), elem.clone()).neg(),
                            SetInference::new(elem.clone()).neg(),
                        ]));
                    }
                    // If an assumption is in the set, it must not be in the attack
                    clauses.push(Clause::from(vec![
                        Inference::new(assumption.clone()).neg(),
                        SetInference::new(assumption.clone()).neg(),
                    ]));
                    // clauses.push(Clause::from(vec![
                    //     Inference::new(assumption.clone()).pos(),
                    //     SetInference::new(assumption.clone()).pos(),
                    // ]))
                }

                clauses
            }
            idx => {
                // If we've found {a, c, d} in the last iteration, prevent it from being picked again
                // Assuming a..=f are our assumptions:
                //   {-a, b, -c, -d, e, f} must be true
                let just_found = &self.found[idx - 1];
                let new_clause = aba
                    .inverses
                    .keys()
                    .map(|assumption| {
                        if just_found.contains(assumption) {
                            SetInference::new(assumption.clone()).neg()
                        } else {
                            SetInference::new(assumption.clone()).pos()
                        }
                    })
                    .collect();
                vec![new_clause]
            }
        }
    }

    fn feedback(
        &mut self,
        aba: &Aba<A>,
        sat_result: bool,
        solver: &cadical::Solver,
        map: &Mapper,
    ) -> LoopControl {
        if !sat_result {
            return LoopControl::Stop;
        }
        // TODO: Somehow query the mapper about things instead of this
        let found = aba
            .inverses
            .keys()
            .filter_map(|assumption| {
                let literal = SetInference::new(assumption.clone()).pos();
                let raw = map.get_raw(&literal)?;
                match solver.value(raw) {
                    Some(true) => Some(assumption.clone()),
                    _ => None,
                }
            })
            .collect();
        eprintln!("Found {found:?}");
        self.found.push(found);
        LoopControl::Continue
    }

    fn construct_output(
        mut self,
        _aba: &Aba<A>,
        _sat_result: bool,
        _solver: &cadical::Solver,
        _total_iterations: usize,
    ) -> Self::Output {
        // Re-Add the empty set
        self.found.push(HashSet::new());
        self.found
    }
}
