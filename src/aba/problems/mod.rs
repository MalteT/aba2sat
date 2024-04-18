use cadical::Solver;

use crate::{
    clauses::ClauseList,
    error::{Error, Result},
    literal::lits::Theory,
    mapper::Mapper,
};

use super::{prepared::PreparedAba, Aba};

pub mod admissibility;
pub mod complete;
pub mod conflict_free;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LoopControl {
    Continue,
    Stop,
}

pub struct SolverState<'a> {
    aba: &'a Aba,
    sat_result: bool,
    solver: &'a Solver,
    map: &'a Mapper,
}

#[doc(notable_trait)]
pub trait Problem {
    type Output;
    fn additional_clauses(&self, aba: &PreparedAba) -> ClauseList;
    fn construct_output(self, state: SolverState<'_>) -> Self::Output;

    fn check(&self, _aba: &Aba) -> Result {
        Ok(())
    }
}

#[doc(notable_trait)]
pub trait MultishotProblem {
    type Output;
    fn additional_clauses(&self, aba: &PreparedAba, iteration: usize) -> ClauseList;
    fn feedback(&mut self, state: SolverState<'_>) -> LoopControl;
    fn construct_output(self, state: SolverState<'_>, total_iterations: usize) -> Self::Output;

    fn check(&self, _aba: &Aba) -> Result {
        Ok(())
    }
}

pub fn solve<P: Problem>(problem: P, aba: Aba) -> Result<P::Output> {
    let aba = aba.prepare();
    // Let the problem perform additional checks before starting the solver
    problem.check(&aba)?;
    // Create a map that will keep track of the translation between
    // atoms as we know them and their SAT representation
    let mut map = Mapper::new();
    // Instantiate a new SAT solver instance
    let mut sat: Solver = Solver::default();
    // Derive clauses from the ABA
    let clauses: ClauseList = aba.derive_clauses::<Theory>().collect();
    // Append additional clauses as defined by the problem
    let additional_clauses = problem.additional_clauses(&aba);
    // Convert the total of our derived clauses using the mapper
    // and feed the solver with the result
    map.as_raw_iter(&clauses)
        .for_each(|raw| sat.add_clause(raw));
    // Do the same with the additional clauses that the problem defined
    map.as_raw_iter(&additional_clauses)
        .for_each(|raw| sat.add_clause(raw));
    // A single solver call to determine the solution
    if let Some(sat_result) = sat.solve() {
        #[cfg(debug_assertions)]
        if sat_result {
            let rec = map.reconstruct(&sat).collect::<Vec<_>>();
            eprintln!("{rec:#?}");
        }
        // If the solver didn't panic, convert our result into the output
        // using our problem instance
        Ok(problem.construct_output(SolverState {
            aba: &aba,
            sat_result,
            solver: &sat,
            map: &map,
        }))
    } else {
        Err(Error::SatCallInterrupted)
    }
}

pub fn multishot_solve<P: MultishotProblem>(mut problem: P, aba: Aba) -> Result<P::Output> {
    let aba = aba.prepare();
    // Let the problem perform additional checks before starting the solver
    problem.check(&aba)?;
    // Create a map that will keep track of the translation between
    // atoms as we know them and their SAT representation
    let mut map = Mapper::new();
    // Instantiate a new SAT solver instance
    let mut sat: Solver = Solver::default();
    // Derive clauses from the ABA
    let clauses: ClauseList = aba.derive_clauses::<Theory>().collect();
    // Convert the total of our derived clauses using the mapper
    // and feed the solver with the result
    map.as_raw_iter(&clauses)
        .for_each(|raw| sat.add_clause(raw));
    // Keep track of the iteration we're in, this is a multishot solve
    let mut iteration = 0;
    // Enter the main loop
    let final_result = loop {
        // Derive additional clauses from the problem instance, these
        // may change for every iteration
        let additional_clauses = problem.additional_clauses(&aba, iteration);
        // Feed the clauses into our mapper and add the output to our running solver instance
        map.as_raw_iter(&additional_clauses)
            .for_each(|raw| sat.add_clause(raw));
        // Call the solver for the next result
        let sat_result = sat.solve().ok_or(Error::SatCallInterrupted)?;
        #[cfg(debug_assertions)]
        if sat_result {
            let rec = map.reconstruct(&sat).collect::<Vec<_>>();
            eprintln!("{rec:#?}");
        }
        // Call our problem to ask whether we should continue. This is the point
        // where the problem instance can exit the loop our mutate inner state
        // with the solver feedback and continue
        let control = problem.feedback(SolverState {
            aba: &aba,
            sat_result,
            solver: &sat,
            map: &map,
        });
        // Exit if the problem instance requested it
        if control == LoopControl::Stop {
            break sat_result;
        }
        // Or continue into the next iteration
        iteration += 1;
    };
    // This point will only be reached if the problem instance
    // is happy with the iterations. Call it one final time to
    // construct the output using the final results
    Ok(problem.construct_output(
        SolverState {
            aba: &aba,
            sat_result: final_result,
            solver: &sat,
            map: &map,
        },
        iteration,
    ))
}
