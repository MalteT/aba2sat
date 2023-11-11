use std::collections::{HashMap, HashSet};

use crate::{
    clauses::{Clause, ClauseList},
    literal::{Inference, InferenceHelper, Inverse},
};

pub mod problems;

#[derive(Debug, Default)]
pub struct Aba {
    rules: Vec<(char, HashSet<char>)>,
    inverses: HashMap<char, char>,
}

impl Aba {
    pub fn new() -> Self {
        Aba {
            rules: vec![],
            inverses: HashMap::new(),
        }
    }

    pub fn with_assumption(mut self, assumption: char, inverse: char) -> Self {
        self.inverses.insert(assumption, inverse);
        self
    }

    pub fn with_rule<B: IntoIterator<Item = char>>(mut self, head: char, body: B) -> Self {
        self.rules.push((head, body.into_iter().collect()));
        self
    }

    pub fn universe(&self) -> impl Iterator<Item = &char> {
        // List of all elements of our ABA, basically our L (universe)
        self.inverses
            .keys()
            .chain(self.inverses.values())
            .chain(self.rules.iter().flat_map(|(_, body)| body))
            .chain(self.rules.iter().map(|(head, _)| head))
    }

    pub fn contains_assumption(&self, a: &char) -> bool {
        self.inverses.contains_key(a)
    }

    /**
     * Translate the ABA into base rules / definitions for SAT solving
     */
    pub fn derive_clauses(&self) -> ClauseList {
        let mut clauses = ClauseList::new();
        self.derive_rule_clauses().collect_into(&mut clauses);
        self.derive_inverse_clauses().collect_into(&mut clauses);
        clauses
    }

    fn derive_rule_clauses(&self) -> impl Iterator<Item = Clause> + '_ {
        inference_helper(&self.rules)
    }

    fn derive_inverse_clauses(&self) -> impl Iterator<Item = Clause> + '_ {
        self.inverses
            .iter()
            .map(|(&from, &to)| Clause::from(vec![lit!(+Inverse :from :to)]))
    }
}

fn body_to_clauses(head: crate::literal::Literal, body: &HashSet<char>) -> ClauseList {
    let mut clauses = vec![];
    let mut left_implication: Clause = body.iter().map(|&elem| lit!(-Inference :elem)).collect();
    left_implication.push(head.clone().positive());
    clauses.push(left_implication);
    body.iter()
        .map(|&elem| vec![head.clone().negative(), lit!(+Inference :elem)].into())
        .collect_into(&mut clauses);
    clauses
}

fn inference_helper(rules: &[(char, HashSet<char>)]) -> impl Iterator<Item = Clause> + '_ {
    let rules_combined =
        rules
            .iter()
            .fold(HashMap::<_, Vec<_>>::new(), |mut rules, (head, body)| {
                rules.entry(head).or_default().push(body);
                rules
            });
    rules_combined
        .into_iter()
        .flat_map(move |(&head, bodies)| match &bodies[..] {
            [] => unreachable!("Heads always have a body"),
            [body] => body_to_clauses(lit!(+Inference elem:head), body),
            bodies => {
                let mut clauses = vec![];
                bodies
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, body)| {
                        body_to_clauses(lit!(+InferenceHelper :idx :head), body)
                    })
                    .collect_into(&mut clauses);
                let helpers: Vec<_> = (0..bodies.len())
                    .map(|idx| lit!(+InferenceHelper :idx :head))
                    .collect();
                let mut right_implification: Clause = helpers.iter().cloned().collect();
                right_implification.push(lit!(-Inference elem:head));
                clauses.push(right_implification);
                helpers
                    .into_iter()
                    .map(|helper| Clause::from(vec![lit!(+Inference elem:head), helper.negative()]))
                    .collect_into(&mut clauses);
                clauses
            }
        })
}
