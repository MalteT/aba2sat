use cadical::Solver;

use crate::{
    aba::Aba,
    clauses::{Clause, ClauseList},
    literal::{Inference, Inverse},
};

use super::Problem;

pub struct ConflictFreeness {
    pub assumptions: Vec<char>,
}

impl Problem for ConflictFreeness {
    type Output = bool;

    fn additional_clauses(&self, aba: &Aba) -> ClauseList {
        let mut clauses = vec![];
        // Make sure that every assumption in our problem is inferred and every other not
        for elem in self.assumptions.iter().copied() {
            if aba.inverses.contains_key(&elem) {
                clauses.push(vec![lit!(+Inference :elem)].into())
            } else {
                clauses.push(vec![lit!(-Inference :elem)].into())
            }
        }
        // TODO: Minimize this loop
        for elem in aba.universe().copied() {
            for assumption in aba.inverses.keys().copied() {
                // For every element e in our universe and every assumption a, we cannot have the following:
                // e is the inverse of a and both are inferred (conflict!)
                clauses.push(Clause::from(vec![
                    lit!(-Inference elem:assumption),
                    lit!(-Inference :elem),
                    lit!(-Inverse from:assumption to:elem),
                ]))
            }
        }
        clauses
    }

    fn construct_output(self, sat_result: bool, _: &Aba, _: &Solver) -> Self::Output {
        sat_result
    }

    fn check(&self, aba: &Aba) -> bool {
        // Make sure that every assumption is part of the ABA
        self.assumptions.iter().all(|a| aba.contains_assumption(a))
    }
}
