use crate::{
    aba::{inference_helper, Aba},
    clauses::{Atom, Clause},
    literal::{Inference, IntoLiteral, Inverse, SetInference},
};

use super::Problem;

pub struct VerifyAdmissibility<A: Atom> {
    pub assumptions: Vec<A>,
}

impl<A: Atom> Problem<A> for VerifyAdmissibility<A> {
    type Output = bool;

    fn additional_clauses(&self, aba: &Aba<A>) -> crate::clauses::ClauseList {
        let mut clauses = vec![];
        // Create inference for the problem set
        inference_helper::<SetInference<_>, _>(&aba.rules).collect_into(&mut clauses);
        // Force inference on all members of the set
        aba.inverses
            .keys()
            .cloned()
            .map(|assumption| {
                if self.assumptions.contains(&assumption) {
                    Clause::from(vec![SetInference::new(assumption).pos()])
                } else {
                    Clause::from(vec![SetInference::new(assumption).neg()])
                }
            })
            .collect_into(&mut clauses);
        // Attack the inference of the aba, if an attack exists
        for elem in aba.universe() {
            for assumption in self.assumptions.iter() {
                clauses.push(Clause::from(vec![
                    SetInference::new(assumption.clone()).neg(),
                    Inverse::new(assumption.clone(), elem.clone()).neg(),
                    Inference::new(elem.clone()).neg(),
                ]))
            }
            for assumption in aba.inverses.keys() {
                clauses.push(Clause::from(vec![
                    SetInference::new(assumption.clone()).neg(),
                    Inverse::new(assumption.clone(), elem.clone()).neg(),
                    SetInference::new(elem.clone()).neg(),
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

    fn check(&self, aba: &Aba<A>) -> bool {
        // Make sure that every assumption is part of the ABA
        self.assumptions.iter().all(|a| aba.contains_assumption(a))
    }
}
