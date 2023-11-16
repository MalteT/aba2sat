use std::collections::{HashMap, HashSet};

use crate::{
    aba::Aba,
    clauses::{Atom, Clause, ClauseList},
    literal::{Inference, IntoLiteral, Inverse, Literal, SetInference, SetInferenceHelper},
};

use super::Problem;

pub struct Admissible<A: Atom> {
    pub assumptions: Vec<A>,
}

impl<A: Atom> Problem<A> for Admissible<A> {
    type Output = bool;

    fn additional_clauses(&self, aba: &Aba<A>) -> crate::clauses::ClauseList {
        let mut clauses = vec![];
        // Create inference for the problem set
        inference_helper(&aba.rules).collect_into(&mut clauses);
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

fn inference_helper<A: Atom>(rules: &[(A, HashSet<A>)]) -> impl Iterator<Item = Clause> + '_ {
    let rules_combined =
        rules
            .iter()
            .fold(HashMap::<_, Vec<_>>::new(), |mut rules, (head, body)| {
                rules.entry(head).or_default().push(body);
                rules
            });
    rules_combined
        .into_iter()
        .flat_map(move |(head, bodies)| match &bodies[..] {
            [] => unreachable!("Heads always have a body"),
            [body] => body_to_clauses(SetInference::new(head.clone()).pos(), body),
            bodies => {
                let mut clauses: Vec<Clause> = vec![];
                bodies
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, body)| {
                        body_to_clauses(SetInferenceHelper::new(idx, head.clone()).pos(), body)
                    })
                    .collect_into(&mut clauses);
                let helpers: Vec<_> = (0..bodies.len())
                    .map(|idx| SetInferenceHelper::new(idx, head.clone()).pos())
                    .collect();
                let mut right_implification: Clause = helpers.iter().cloned().collect();
                right_implification.push(SetInference::new(head.clone()).neg());
                clauses.push(right_implification);
                helpers
                    .into_iter()
                    .map(|helper| {
                        Clause::from(vec![
                            SetInference::new(head.clone()).pos(),
                            helper.negative(),
                        ])
                    })
                    .collect_into(&mut clauses);
                clauses
            }
        })
}

fn body_to_clauses<A: Atom>(head: Literal, body: &HashSet<A>) -> ClauseList {
    let mut clauses = vec![];
    let mut left_implication: Clause = body
        .iter()
        .map(|elem| SetInference::new(elem.clone()).neg())
        .collect();
    left_implication.push(head.clone().positive());
    clauses.push(left_implication);
    body.iter()
        .map(|elem| {
            vec![
                head.clone().negative(),
                SetInference::new(elem.clone()).pos(),
            ]
            .into()
        })
        .collect_into(&mut clauses);
    clauses
}
