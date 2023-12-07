use cadical::Solver;

use crate::{
    clauses::{Atom, ClauseList},
    error::{Error, Result},
    literal::TheoryAtom,
    mapper::Mapper,
};

use super::Aba;

pub mod admissibility;
pub mod complete;
pub mod conflict_free;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LoopControl {
    Continue,
    Stop,
}

pub struct SolverState<'a, A: Atom + 'a> {
    aba: &'a Aba<A>,
    sat_result: bool,
    solver: &'a Solver,
    map: &'a Mapper,
}

#[doc(notable_trait)]
pub trait Problem<A: Atom> {
    type Output;
    fn additional_clauses(&self, aba: &Aba<A>) -> ClauseList;
    fn construct_output(self, state: SolverState<'_, A>) -> Self::Output;

    fn check(&self, _aba: &Aba<A>) -> Result {
        Ok(())
    }
}

#[doc(notable_trait)]
pub trait MultishotProblem<A: Atom> {
    type Output;
    fn additional_clauses(&self, aba: &Aba<A>, iteration: usize) -> ClauseList;
    fn feedback(&mut self, state: SolverState<'_, A>) -> LoopControl;
    fn construct_output(self, state: SolverState<'_, A>, total_iterations: usize) -> Self::Output;

    fn check(&self, _aba: &Aba<A>) -> Result {
        Ok(())
    }
}

/// *(Literal)* `A` is element of `th(S)`
#[derive(Debug)]
pub struct SetTheory<A: Atom>(A);

/// Helper for [`SetTheory`]
#[derive(Debug)]
pub struct SetTheoryHelper<A: Atom>(usize, A);

pub fn solve<A: Atom, P: Problem<A>>(problem: P, aba: &Aba<A>) -> Result<P::Output> {
    problem.check(aba)?;
    let clauses = aba.derive_clauses();
    let additional_clauses = problem.additional_clauses(aba);
    let mut map = Mapper::new();
    let mut sat: Solver = Solver::default();
    map.as_raw_iter(&clauses)
        .for_each(|raw| sat.add_clause(raw));
    map.as_raw_iter(&additional_clauses)
        .for_each(|raw| sat.add_clause(raw));
    if let Some(sat_result) = sat.solve() {
        Ok(problem.construct_output(SolverState {
            aba,
            sat_result,
            solver: &sat,
            map: &map,
        }))
    } else {
        Err(Error::SatCallInterrupted)
    }
}

pub fn multishot_solve<A: Atom, P: MultishotProblem<A>>(
    mut problem: P,
    aba: &Aba<A>,
) -> Result<P::Output> {
    problem.check(aba)?;
    let mut map = Mapper::new();
    let mut sat: Solver = Solver::default();
    let mut iteration = 0;
    let clauses = aba.derive_clauses();
    map.as_raw_iter(&clauses)
        .for_each(|raw| sat.add_clause(raw));
    let final_result = loop {
        let additional_clauses = problem.additional_clauses(aba, iteration);
        map.as_raw_iter(&additional_clauses)
            .for_each(|raw| sat.add_clause(raw));
        let sat_result = sat.solve().ok_or(Error::SatCallInterrupted)?;
        #[cfg(debug_assertions)]
        if sat_result {
            let rec = map.reconstruct(&sat).collect::<Vec<_>>();
            eprintln!("{rec:#?}");
        }
        let control = problem.feedback(SolverState {
            aba,
            sat_result,
            solver: &sat,
            map: &map,
        });
        if control == LoopControl::Stop {
            break sat_result;
        }
        iteration += 1;
    };
    Ok(problem.construct_output(
        SolverState {
            aba,
            sat_result: final_result,
            solver: &sat,
            map: &map,
        },
        iteration,
    ))
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
