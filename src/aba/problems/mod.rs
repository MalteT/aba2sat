use cadical::Solver;

use crate::{
    clauses::{Atom, ClauseList},
    error::{Error, Result},
    mapper::Mapper,
};

use super::Aba;

pub mod admissibility;
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

pub trait Problem<A: Atom> {
    type Output;
    fn additional_clauses(&self, aba: &Aba<A>) -> ClauseList;
    fn construct_output(self, state: SolverState<'_, A>) -> Self::Output;

    fn check(&self, _aba: &Aba<A>) -> bool {
        true
    }
}

pub trait MultishotProblem<A: Atom> {
    type Output;
    fn additional_clauses(&self, aba: &Aba<A>, iteration: usize) -> ClauseList;
    fn feedback(&mut self, state: SolverState<'_, A>) -> LoopControl;
    fn construct_output(self, state: SolverState<'_, A>, total_iterations: usize) -> Self::Output;

    fn check(&self, _aba: &Aba<A>) -> bool {
        true
    }
}

pub fn solve<A: Atom, P: Problem<A>>(problem: P, aba: &Aba<A>) -> Result<P::Output> {
    if problem.check(aba) {
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
    } else {
        Err(Error::ProblemCheckFailed)
    }
}

pub fn multishot_solve<A: Atom, P: MultishotProblem<A>>(
    mut problem: P,
    aba: &Aba<A>,
) -> Result<P::Output> {
    if !problem.check(aba) {
        return Err(Error::ProblemCheckFailed);
    }
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
