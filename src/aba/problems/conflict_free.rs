use std::collections::HashSet;

use crate::{
    aba::{Aba, Theory},
    clauses::{Atom, Clause, ClauseList},
    error::Error,
    literal::{IntoLiteral, TheoryAtom},
    Result,
};

use super::{Problem, SolverState};

pub struct ConflictFreeness<A: Atom> {
    pub assumptions: HashSet<A>,
}

impl<A: Atom> Problem<A> for ConflictFreeness<A> {
    type Output = bool;

    fn additional_clauses(&self, aba: &Aba<A>) -> ClauseList {
        let mut clauses = vec![];
        // Make sure that every assumption in our problem is inferred and every other not
        for assumption in aba.assumptions() {
            let theory = Theory::new(assumption.clone());
            if self.assumptions.contains(assumption) {
                clauses.push(vec![theory.pos()].into())
            } else {
                clauses.push(vec![theory.neg()].into())
            }
        }
        for (assumption, inverse) in &aba.inverses {
            clauses.push(Clause::from(vec![
                Theory::new(assumption.clone()).neg(),
                Theory::new(inverse.clone()).neg(),
            ]));
        }
        clauses
    }

    fn construct_output(self, state: SolverState<'_, A>) -> Self::Output {
        state.sat_result
    }

    fn check(&self, aba: &Aba<A>) -> Result {
        // Make sure that every assumption is part of the ABA
        match self
            .assumptions
            .iter()
            .find(|a| !aba.contains_assumption(a))
        {
            Some(assumption) => Err(Error::ProblemCheckFailed(format!(
                "Assumption {:?} not present in ABA framework",
                assumption
            ))),
            None => Ok(()),
        }
    }
}
