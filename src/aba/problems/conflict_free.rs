use std::collections::HashSet;

use crate::{
    aba::{prepared::PreparedAba, Aba, Num},
    clauses::{Clause, ClauseList},
    error::Error,
    literal::{lits::Theory, IntoLiteral},
    Result,
};

use super::{Problem, SolverState};

pub struct ConflictFreeness {
    pub assumptions: HashSet<Num>,
}

impl Problem for ConflictFreeness {
    type Output = bool;

    fn additional_clauses(&self, aba: &PreparedAba) -> ClauseList {
        let mut clauses = vec![];
        // Make sure that every assumption in our problem is inferred and every other not
        for assumption in aba.assumptions() {
            let theory = Theory::from(*assumption);
            if self.assumptions.contains(assumption) {
                clauses.push(vec![theory.pos()].into())
            } else {
                clauses.push(vec![theory.neg()].into())
            }
        }
        for (assumption, inverse) in &aba.inverses {
            clauses.push(Clause::from(vec![
                Theory::from(*assumption).neg(),
                Theory::from(*inverse).neg(),
            ]));
        }
        clauses
    }

    fn construct_output(self, state: SolverState<'_>) -> Self::Output {
        state.sat_result
    }

    fn check(&self, aba: &Aba) -> Result {
        // Make sure that every assumption is part of the ABA
        match self
            .assumptions
            .iter()
            .find(|assumption| !aba.contains_assumption(assumption))
        {
            Some(assumption) => Err(Error::ProblemCheckFailed(format!(
                "Assumption {:?} not present in ABA framework",
                assumption
            ))),
            None => Ok(()),
        }
    }
}
