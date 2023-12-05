use cadical::Solver;

use crate::{
    aba::{Aba, Inference, Inverse},
    clauses::{Clause, ClauseList},
    literal::{InferenceAtom, IntoLiteral},
};

use super::Problem;

pub struct ConflictFreeness {
    pub assumptions: Vec<char>,
}

impl Problem<char> for ConflictFreeness {
    type Output = bool;

    fn additional_clauses(&self, aba: &Aba<char>) -> ClauseList {
        let mut clauses = vec![];
        // Make sure that every assumption in our problem is inferred and every other not
        for elem in self.assumptions.iter().copied() {
            if aba.inverses.contains_key(&elem) {
                clauses.push(vec![Inference::new(elem).pos()].into())
            } else {
                clauses.push(vec![Inference::new(elem).neg()].into())
            }
        }
        // TODO: Minimize this loop
        for elem in aba.universe().copied() {
            for assumption in aba.assumptions().copied() {
                // For every element e in our universe and every assumption a, we cannot have the following:
                // e is the inverse of a and both are inferred (conflict!)
                clauses.push(Clause::from(vec![
                    Inference::new(assumption).neg(),
                    Inference::new(elem).neg(),
                    Inverse {
                        from: assumption,
                        to: elem,
                    }
                    .neg(),
                ]))
            }
        }
        clauses
    }

    fn construct_output(self, sat_result: bool, _: &Aba<char>, _: &Solver) -> Self::Output {
        sat_result
    }

    fn check(&self, aba: &Aba<char>) -> bool {
        // Make sure that every assumption is part of the ABA
        self.assumptions.iter().all(|a| aba.contains_assumption(a))
    }
}
