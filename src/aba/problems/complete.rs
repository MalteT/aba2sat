use std::collections::HashSet;

use crate::{
    aba::{Aba, Num, Theory},
    clauses::{Clause, ClauseList},
    error::Error,
    literal::{IntoLiteral, TheoryAtom},
    Result,
};

use super::{
    admissibility::initial_admissibility_clauses, LoopControl, MultishotProblem, Problem,
    SetTheory, SolverState,
};

#[derive(Debug, Default)]
pub struct EnumerateCompleteExtensions {
    found: Vec<HashSet<Num>>,
}

/// Decide whether `assumption` is credulously complete in an [`Aba`]
pub struct DecideCredulousComplete {
    pub element: Num,
}

fn initial_complete_clauses(aba: &Aba) -> ClauseList {
    // Take everything from admissibility
    let mut clauses = initial_admissibility_clauses::<SetTheory>(aba);
    // Additional complete logic
    for (assumption, inverse) in &aba.inverses {
        // For any assumption `a` and it's inverse `b`:
        //   b not in th(A) => a in th(S)
        clauses.push(Clause::from(vec![
            Theory(*inverse).pos(),
            SetTheory(*assumption).pos(),
        ]));
    }
    clauses
}

impl MultishotProblem<Num> for EnumerateCompleteExtensions {
    type Output = Vec<HashSet<Num>>;

    fn additional_clauses(&self, aba: &Aba, iteration: usize) -> ClauseList {
        match iteration {
            0 => initial_complete_clauses(aba),
            idx => {
                // If we've found {a, c, d} in the last iteration, prevent it from being picked again
                // Assuming a..=f are our assumptions:
                //   {-a, b, -c, -d, e, f} must be true
                let just_found = &self.found[idx - 1];
                let new_clause = aba
                    .assumptions()
                    .map(|assumption| {
                        if just_found.contains(assumption) {
                            SetTheory::new(*assumption).neg()
                        } else {
                            SetTheory::new(*assumption).pos()
                        }
                    })
                    .collect();
                vec![new_clause]
            }
        }
    }

    fn feedback(&mut self, state: SolverState<'_>) -> LoopControl {
        if !state.sat_result {
            return LoopControl::Stop;
        }
        // TODO: Somehow query the mapper about things instead of this
        let found = state
            .aba
            .inverses
            .keys()
            .filter_map(|assumption| {
                let literal = SetTheory::new(*assumption).pos();
                let raw = state.map.get_raw(&literal)?;
                match state.solver.value(raw) {
                    Some(true) => Some(*assumption),
                    _ => None,
                }
            })
            .collect();
        self.found.push(found);
        LoopControl::Continue
    }

    fn construct_output(self, _state: SolverState<'_>, _total_iterations: usize) -> Self::Output {
        self.found
    }
}

impl Problem for DecideCredulousComplete {
    type Output = bool;

    fn additional_clauses(&self, aba: &Aba) -> ClauseList {
        let mut clauses = initial_complete_clauses(aba);
        clauses.push(Clause::from(vec![SetTheory(self.element).pos()]));
        clauses
    }

    fn construct_output(self, state: SolverState<'_>) -> Self::Output {
        state.sat_result
    }

    fn check(&self, aba: &Aba) -> Result {
        if aba.contains_assumption(&self.element) {
            Ok(())
        } else {
            Err(Error::ProblemCheckFailed(format!(
                "Element {:?} not present in ABA framework",
                self.element
            )))
        }
    }
}
