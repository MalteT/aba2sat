use crate::{
    aba::{inference_helper, Aba},
    clauses::{Atom, Clause},
    literal::{Inference, IntoLiteral, Inverse, SetInference},
};

use super::Problem;

pub struct Admissibility;

impl<A: Atom> Problem<A> for Admissibility {
    type Output = bool;

    fn additional_clauses(&self, aba: &Aba<A>) -> crate::clauses::ClauseList {
        let mut clauses = vec![];
        // Create inference for the problem set
        inference_helper::<SetInference<_>, _>(&aba.rules).collect_into(&mut clauses);
        // Prevent the empty set
        let no_empty_set: Clause = aba
            .inverses
            .keys()
            .map(|assumption| SetInference::new(assumption.clone()).pos())
            .collect();
        clauses.push(no_empty_set);
        // Attack the inference of the aba, if an attack exists
        for elem in aba.universe() {
            for assumption in aba.inverses.keys() {
                clauses.push(Clause::from(vec![
                    SetInference::new(assumption.clone()).neg(),
                    Inverse::new(assumption.clone(), elem.clone()).neg(),
                    SetInference::new(elem.clone()).neg(),
                ]));
                clauses.push(Clause::from(vec![
                    SetInference::new(assumption.clone()).neg(),
                    Inverse::new(assumption.clone(), elem.clone()).neg(),
                    Inference::new(elem.clone()).neg(),
                ]))
            }
        }

        clauses
    }

    fn construct_output(
        self,
        sat_result: bool,
        _: &crate::aba::Aba<A>,
        _: &cadical::Solver,
    ) -> Self::Output {
        sat_result
    }
}
