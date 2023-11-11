use std::collections::{HashMap, HashSet};

use crate::{
    clauses::{Clause, ClauseList},
    literal::{SetInference, SetInferenceHelper},
};

use super::Problem;

pub struct Admissible {
    pub assumptions: Vec<char>,
}

impl Problem for Admissible {
    type Output = bool;

    fn additional_clauses(&self, aba: &crate::aba::Aba) -> crate::clauses::ClauseList {
        let mut clauses = vec![];
        // Create inference for the problem set
        inference_helper(&aba.rules).collect_into(&mut clauses);
        aba.inverses
            .keys()
            .copied()
            .map(|assumption| {
                if self.assumptions.contains(&assumption) {
                    Clause::from(vec![lit!(+SetInference elem:assumption)])
                } else {
                    Clause::from(vec![lit!(-SetInference elem:assumption)])
                }
            })
            .collect_into(&mut clauses);
        // TODO: Attack the inference of the aba, if an attack exists

        clauses
    }

    fn construct_output(
        self,
        sat_result: bool,
        _: &crate::aba::Aba,
        _: &cadical::Solver,
    ) -> Self::Output {
        sat_result
    }
}

fn inference_helper(rules: &[(char, HashSet<char>)]) -> impl Iterator<Item = Clause> + '_ {
    let rules_combined = rules
        .iter()
        .fold(HashMap::new(), |mut rules, (head, body)| {
            rules.entry(head).or_insert(vec![]).push(body);
            rules
        });
    rules_combined
        .into_iter()
        .flat_map(move |(&head, bodies)| match &bodies[..] {
            [] => unreachable!("Heads always have a body"),
            [body] => body_to_clauses(lit!(+SetInference elem:head), body),
            bodies => {
                let mut clauses = vec![];
                bodies
                    .into_iter()
                    .enumerate()
                    .flat_map(|(idx, body)| {
                        body_to_clauses(lit!(+SetInferenceHelper :idx :head), body)
                    })
                    .collect_into(&mut clauses);
                let helpers: Vec<_> = (0..bodies.len())
                    .map(|idx| lit!(+SetInferenceHelper :idx :head))
                    .collect();
                let mut right_implification: Clause = helpers.iter().cloned().collect();
                right_implification.push(lit!(-SetInference elem:head));
                clauses.push(right_implification);
                helpers
                    .into_iter()
                    .map(|helper| {
                        Clause::from(vec![lit!(+SetInference elem:head), helper.negative()])
                    })
                    .collect_into(&mut clauses);
                clauses
            }
        })
}

fn body_to_clauses(head: crate::literal::Literal, body: &HashSet<char>) -> ClauseList {
    let mut clauses = vec![];
    let mut left_implication: Clause = body.iter().map(|&elem| lit!(-SetInference :elem)).collect();
    left_implication.push(head.clone().positive());
    clauses.push(left_implication);
    body.iter()
        .map(|&elem| vec![head.clone().negative(), lit!(+SetInference :elem)].into())
        .collect_into(&mut clauses);
    clauses
}
