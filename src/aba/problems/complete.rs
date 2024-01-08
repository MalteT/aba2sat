use std::collections::HashSet;

use crate::{
    aba::{Aba, Theory},
    clauses::{Atom, Clause, ClauseList},
    error::Error,
    literal::{IntoLiteral, TheoryAtom},
    Result,
};

use super::{
    admissibility::initial_admissibility_clauses, LoopControl, MultishotProblem, Problem,
    SetTheory, SolverState,
};

#[derive(Debug, Default)]
pub struct EnumerateCompleteExtensions<A> {
    found: Vec<HashSet<A>>,
}

/// Decide whether `assumption` is credulously complete in an [`Aba`]
pub struct DecideCredulousComplete<A> {
    pub element: A,
}

fn initial_complete_clauses<A: Atom>(aba: &Aba<A>) -> ClauseList {
    // Take everything from admissibility
    let mut clauses = initial_admissibility_clauses::<SetTheory<_>, _>(aba);
    // Additional complete logic
    for (assumption, inverse) in &aba.inverses {
        // For any assumption `a` and it's inverse `b`:
        //   b not in th(A) => a in th(S)
        clauses.push(Clause::from(vec![
            Theory(inverse.clone()).pos(),
            SetTheory(assumption.clone()).pos(),
        ]));
    }
    clauses
}

impl<A: Atom> MultishotProblem<A> for EnumerateCompleteExtensions<A> {
    type Output = Vec<HashSet<A>>;

    fn additional_clauses(&self, aba: &Aba<A>, iteration: usize) -> ClauseList {
        match iteration {
            0 => initial_complete_clauses(aba),
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
                            SetTheory::new(assumption.clone()).neg()
                        } else {
                            SetTheory::new(assumption.clone()).pos()
                        }
                    })
                    .collect();
                vec![new_clause]
            }
        }
    }

    fn feedback(&mut self, state: SolverState<'_, A>) -> LoopControl {
        if !state.sat_result {
            return LoopControl::Stop;
        }
        // TODO: Somehow query the mapper about things instead of this
        let found = state
            .aba
            .inverses
            .keys()
            .filter_map(|assumption| {
                let literal = SetTheory::new(assumption.clone()).pos();
                let raw = state.map.get_raw(&literal)?;
                match state.solver.value(raw) {
                    Some(true) => Some(assumption.clone()),
                    _ => None,
                }
            })
            .collect();
        self.found.push(found);
        LoopControl::Continue
    }

    fn construct_output(
        self,
        _state: SolverState<'_, A>,
        _total_iterations: usize,
    ) -> Self::Output {
        self.found
    }
}

impl<A: Atom> Problem<A> for DecideCredulousComplete<A> {
    type Output = bool;

    fn additional_clauses(&self, aba: &Aba<A>) -> ClauseList {
        let mut clauses = initial_complete_clauses(aba);
        clauses.push(Clause::from(vec![SetTheory(self.element.clone()).pos()]));
        clauses
    }

    fn construct_output(self, state: SolverState<'_, A>) -> Self::Output {
        state.sat_result
    }

    fn check(&self, aba: &Aba<A>) -> Result {
        if aba.has_element(&self.element) {
            Ok(())
        } else {
            Err(Error::ProblemCheckFailed(format!(
                "Element {:?} not present in ABA framework",
                self.element
            )))
        }
    }
}
