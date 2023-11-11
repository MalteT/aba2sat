use cadical::Solver;

use crate::{clauses::ClauseList, mapper::Mapper};

use super::Aba;

mod admissible;
mod conflict_free;

pub use admissible::Admissible;
pub use conflict_free::ConflictFreeness;

pub trait Problem {
    type Output;
    fn additional_clauses(&self, aba: &Aba) -> ClauseList;
    fn construct_output(self, sat_result: bool, aba: &Aba, solver: &Solver) -> Self::Output;

    fn check(&self, _aba: &Aba) -> bool {
        true
    }
}

pub fn solve<P: Problem>(problem: P, aba: &Aba) -> P::Output {
    if problem.check(aba) {
        let clauses = aba.derive_clauses();
        eprintln!("Clauses from ABA: {clauses:#?}");
        let additional_clauses = problem.additional_clauses(aba);
        eprintln!("Clauses from Problem: {additional_clauses:#?}");
        let mut map = Mapper::new();
        let mut sat: Solver = Solver::default();
        map.as_raw_iter(&clauses)
            .for_each(|raw| sat.add_clause(raw));
        map.as_raw_iter(&additional_clauses)
            .for_each(|raw| sat.add_clause(raw));
        if let Some(sat_result) = sat.solve() {
            eprintln!("=> {sat_result}");
            if sat_result {
                eprintln!("{:#?}", map.reconstruct(&sat).collect::<Vec<_>>());
            }
            problem.construct_output(sat_result, aba, &sat)
        } else {
            unimplemented!("What to do if the solve failed?")
        }
    } else {
        unimplemented!("What to do for an invalid check?")
    }
}
