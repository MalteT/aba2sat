//! Everything needed to solve problems around the admissibility semantics.
use std::collections::HashSet;

use crate::{
    aba::{prepared::PreparedAba, Aba, Num},
    clauses::{Clause, ClauseList},
    error::Error,
    literal::{
        lits::{Theory, TheorySet},
        IntoLiteral,
    },
    Result,
};

use super::{LoopControl, MultishotProblem, Problem, SolverState};

/// Compute all admissible extensions for an [`Aba`]
#[derive(Default, Debug)]
pub struct EnumerateAdmissibleExtensions {
    found: Vec<HashSet<Num>>,
}

/// Sample an admissible extensions from an [`Aba`].
/// Will only return the empty set if no other extension is found
#[derive(Debug, Default)]
pub struct SampleAdmissibleExtension;

/// Verify wether `assumptions` is an admissible extension of an [`Aba`]
pub struct VerifyAdmissibleExtension {
    pub assumptions: HashSet<Num>,
}

/// Decide whether `assumption` is credulously admissible in an [`Aba`]
pub struct DecideCredulousAdmissibility {
    pub element: Num,
}

pub fn initial_admissibility_clauses(aba: &PreparedAba) -> ClauseList {
    let mut clauses = vec![];
    // Create inference for the problem set
    aba.derive_clauses::<TheorySet>().collect_into(&mut clauses);
    // Attack the inference of the aba, if an attack exists
    for (assumption, inverse) in &aba.inverses {
        [
            // For any assumption `a` and it's inverse `b`:
            //   a in th(A) <=> b not in th(S)
            Clause::from(vec![
                Theory::from(*assumption).pos(),
                TheorySet::from(*inverse).pos(),
            ]),
            Clause::from(vec![
                Theory::from(*assumption).neg(),
                TheorySet::from(*inverse).neg(),
            ]),
            // Prevent attacks from the opponent to the selected set
            // For any assumption `a` and it's inverse `b`:
            //   b in th(A) and a in th(S) => bottom
            Clause::from(vec![
                Theory::from(*inverse).neg(),
                TheorySet::from(*assumption).neg(),
            ]),
            // Prevent self-attacks
            // For any assumption `a` and it's inverse `b`:
            //   a in th(S) and b in th(S) => bottom
            Clause::from(vec![
                TheorySet::from(*assumption).neg(),
                TheorySet::from(*inverse).neg(),
            ]),
        ]
        .into_iter()
        .collect_into(&mut clauses);
    }
    clauses
}

fn construct_found_set(state: SolverState<'_>) -> HashSet<Num> {
    state
        .aba
        .assumptions()
        .filter_map(|assumption| {
            let literal = TheorySet::from(*assumption).pos();
            let raw = state.map.get_raw(&literal)?;
            match state.solver.value(raw) {
                Some(true) => Some(*assumption),
                _ => None,
            }
        })
        .collect()
}

impl Problem for SampleAdmissibleExtension {
    type Output = HashSet<Num>;

    fn additional_clauses(&self, aba: &PreparedAba) -> ClauseList {
        let mut clauses = initial_admissibility_clauses(aba);
        // Prevent the empty set
        let no_empty_set: Clause = aba
            .inverses
            .keys()
            .map(|assumption| TheorySet::from(*assumption).pos())
            .collect();
        clauses.push(no_empty_set);
        clauses
    }

    fn construct_output(self, state: SolverState<'_>) -> Self::Output {
        if state.sat_result {
            construct_found_set(state)
        } else {
            HashSet::new()
        }
    }
}

impl MultishotProblem for EnumerateAdmissibleExtensions {
    type Output = Vec<HashSet<Num>>;

    fn additional_clauses(&self, aba: &PreparedAba, iteration: usize) -> ClauseList {
        match iteration {
            0 => {
                let mut clauses = initial_admissibility_clauses(aba);
                // Prevent the empty set
                let no_empty_set: Clause = aba
                    .inverses
                    .keys()
                    .map(|assumption| TheorySet::from(*assumption).pos())
                    .collect();
                clauses.push(no_empty_set);
                clauses
            }
            idx => {
                // If we've found {a, c, d} in the last iteration, prevent it from being picked again
                // Assuming a..=f are our assumptions:
                //   {-a, b, -c, -d, e, f} must be true
                let just_found = &self.found[idx - 1];
                let new_clause = aba
                    .assumptions()
                    .map(|assumption| {
                        if just_found.contains(assumption) {
                            TheorySet::from(*assumption).neg()
                        } else {
                            TheorySet::from(*assumption).pos()
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
                let literal = TheorySet::from(*assumption).pos();
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

    fn construct_output(
        mut self,
        _state: SolverState<'_>,
        _total_iterations: usize,
    ) -> Self::Output {
        // Re-Add the empty set
        self.found.push(HashSet::new());
        self.found
            .into_iter()
            .map(|set| set.into_iter().collect())
            .collect()
    }
}

impl Problem for VerifyAdmissibleExtension {
    type Output = bool;

    fn additional_clauses(&self, aba: &PreparedAba) -> crate::clauses::ClauseList {
        let mut clauses = initial_admissibility_clauses(aba);
        // Force inference on all members of the set
        for assumption in aba.assumptions() {
            let inf = TheorySet::from(*assumption);
            if self.assumptions.contains(assumption) {
                clauses.push(Clause::from(vec![inf.pos()]))
            } else {
                clauses.push(Clause::from(vec![inf.neg()]))
            }
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
                "Assumption {assumption:?} not present in ABA framework"
            ))),
            None => Ok(()),
        }
    }
}

impl Problem for DecideCredulousAdmissibility {
    type Output = bool;

    fn additional_clauses(&self, aba: &PreparedAba) -> ClauseList {
        let mut clauses = initial_admissibility_clauses(aba);
        clauses.push(Clause::from(vec![TheorySet::from(self.element).pos()]));
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
                "Assumption {:?} not present in ABA framework",
                self.element
            )))
        }
    }
}
