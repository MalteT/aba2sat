//! Everything needed to solve problems around the admissibility semantics.
use std::collections::HashSet;

use crate::{
    aba::{inference_helper, Aba, Theory},
    clauses::{Atom, Clause, ClauseList},
    literal::{IntoLiteral, TheoryAtom},
};

use super::{LoopControl, MultishotProblem, Problem, SolverState};

/// Compute all admissible extensions for an [`Aba`]
#[derive(Default, Debug)]
pub struct EnumerateAdmissibleExtensions<A: Atom> {
    found: Vec<HashSet<A>>,
}

/// Sample an admissible extensions from an [`Aba`].
/// Will only return the empty set if no other extension is found
#[derive(Debug, Default)]
pub struct SampleAdmissibleExtension;

/// Verify wether `assumptions` is an admissible extension of an [`Aba`]
pub struct VerifyAdmissibleExtension<A: Atom> {
    pub assumptions: Vec<A>,
}

/// *(Literal)* `A` is element of `th(S)`
#[derive(Debug)]
pub struct SetTheory<A: Atom>(A);

/// Helper for [`SetTheory`]
#[derive(Debug)]
pub struct SetTheoryHelper<A: Atom>(usize, A);

fn initial_clauses<A: Atom>(aba: &Aba<A>) -> ClauseList {
    let mut clauses = vec![];
    // Create inference for the problem set
    inference_helper::<SetTheory<_>, _>(aba).collect_into(&mut clauses);
    // Attack the inference of the aba, if an attack exists
    for (assumption, inverse) in &aba.inverses {
        [
            // For any assumption `a` and it's inverse `b`:
            //   Inference(a) <=> not SetInference(b)
            Clause::from(vec![
                Theory::new(assumption.clone()).pos(),
                SetTheory::new(inverse.clone()).pos(),
            ]),
            Clause::from(vec![
                Theory::new(assumption.clone()).neg(),
                SetTheory::new(inverse.clone()).neg(),
            ]),
            // Prevent attacks from the opponent to the selected set
            // For any assumption `a` and it's inverse `b`:
            //   Inference(b) and SetInference(a) => bottom
            Clause::from(vec![
                Theory::new(inverse.clone()).neg(),
                SetTheory::new(assumption.clone()).neg(),
            ]),
            // Prevent self-attacks
            // For any assumption `a` and it's inverse `b`:
            //   SetInference(a) and SetInference(b) => bottom
            Clause::from(vec![
                SetTheory::new(assumption.clone()).neg(),
                SetTheory::new(inverse.clone()).neg(),
            ]),
        ]
        .into_iter()
        .collect_into(&mut clauses);
    }

    clauses
}

fn construct_found_set<A: Atom>(state: SolverState<'_, A>) -> HashSet<A> {
    state
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
        .collect()
}

impl<A: Atom> Problem<A> for SampleAdmissibleExtension {
    type Output = HashSet<A>;

    fn additional_clauses(&self, aba: &Aba<A>) -> ClauseList {
        let mut clauses = initial_clauses(aba);
        // Prevent the empty set
        let no_empty_set: Clause = aba
            .inverses
            .keys()
            .map(|assumption| SetTheory::new(assumption.clone()).pos())
            .collect();
        clauses.push(no_empty_set);
        clauses
    }

    fn construct_output(self, state: SolverState<'_, A>) -> Self::Output {
        if state.sat_result {
            construct_found_set(state)
        } else {
            HashSet::new()
        }
    }
}

impl<A: Atom> MultishotProblem<A> for EnumerateAdmissibleExtensions<A> {
    type Output = Vec<HashSet<A>>;

    fn additional_clauses(&self, aba: &Aba<A>, iteration: usize) -> ClauseList {
        match iteration {
            0 => {
                let mut clauses = initial_clauses(aba);
                // Prevent the empty set
                let no_empty_set: Clause = aba
                    .inverses
                    .keys()
                    .map(|assumption| SetTheory::new(assumption.clone()).pos())
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
        mut self,
        _state: SolverState<'_, A>,
        _total_iterations: usize,
    ) -> Self::Output {
        // Re-Add the empty set
        self.found.push(HashSet::new());
        self.found
    }
}

impl<A: Atom> Problem<A> for VerifyAdmissibleExtension<A> {
    type Output = bool;

    fn additional_clauses(&self, aba: &Aba<A>) -> crate::clauses::ClauseList {
        let mut clauses = initial_clauses(aba);
        // Force inference on all members of the set
        for assumption in aba.assumptions() {
            let inf = SetTheory::new(assumption.clone());
            if self.assumptions.contains(assumption) {
                clauses.push(Clause::from(vec![inf.pos()]))
            } else {
                clauses.push(Clause::from(vec![inf.neg()]))
            }
        }
        clauses
    }

    fn construct_output(self, state: SolverState<'_, A>) -> Self::Output {
        state.sat_result
    }

    fn check(&self, aba: &Aba<A>) -> bool {
        // Make sure that every assumption is part of the ABA
        self.assumptions.iter().all(|a| aba.contains_assumption(a))
    }
}

impl<A: Atom> TheoryAtom<A> for SetTheory<A> {
    type Helper = SetTheoryHelper<A>;

    fn new(atom: A) -> Self {
        Self(atom)
    }

    fn new_helper(idx: usize, atom: A) -> Self::Helper {
        SetTheoryHelper(idx, atom)
    }
}
