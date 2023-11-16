use std::collections::{HashMap, HashSet};

use crate::{
    clauses::{Atom, Clause, ClauseList},
    literal::{Inference, InferenceHelper, IntoLiteral, Inverse, Literal},
};

pub mod problems;

#[derive(Debug, Default, PartialEq)]
pub struct Aba<A: Atom> {
    pub rules: Vec<(A, HashSet<A>)>,
    pub inverses: HashMap<A, A>,
}

impl<A: Atom> Aba<A> {
    pub fn new() -> Self {
        Aba {
            rules: vec![],
            inverses: HashMap::new(),
        }
    }

    pub fn with_assumption(mut self, assumption: A, inverse: A) -> Self {
        self.inverses.insert(assumption, inverse);
        self
    }

    pub fn with_rule<B: IntoIterator<Item = A>>(mut self, head: A, body: B) -> Self {
        self.rules.push((head, body.into_iter().collect()));
        self
    }

    pub fn universe(&self) -> impl Iterator<Item = &A> {
        // List of all elements of our ABA, basically our L (universe)
        self.inverses
            .keys()
            .chain(self.inverses.values())
            .chain(self.rules.iter().flat_map(|(_, body)| body))
            .chain(self.rules.iter().map(|(head, _)| head))
    }

    pub fn contains_assumption(&self, a: &A) -> bool {
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

    pub fn size(&self) -> usize {
        let inverses = self
            .inverses
            .iter()
            .flat_map(|(assumption, inverse)| [assumption, inverse]);
        self.rules
            .iter()
            .flat_map(|(key, rules)| ::std::iter::once(key).chain(rules))
            .chain(inverses)
            .collect::<HashSet<_>>()
            .len()
    }

    fn derive_rule_clauses(&self) -> impl Iterator<Item = Clause> + '_ {
        inference_helper(&self.rules)
    }

    fn derive_inverse_clauses(&self) -> impl Iterator<Item = Clause> + '_ {
        self.inverses.iter().map(|(from, to)| {
            let inverse: Inverse<A> = Inverse {
                from: from.clone(),
                to: to.clone(),
            };
            Clause::from(vec![inverse.pos()])
        })
    }
}

fn body_to_clauses<A: Atom>(head: Literal, body: &HashSet<A>) -> ClauseList {
    let mut clauses = vec![];
    let mut left_implication: Clause = body
        .iter()
        .map(|elem| Inference::new(elem.clone()).neg())
        .collect();
    left_implication.push(head.clone().positive());
    clauses.push(left_implication);
    body.iter()
        .map(|elem| vec![head.clone().negative(), Inference::new(elem.clone()).pos()].into())
        .collect_into(&mut clauses);
    clauses
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
        .flat_map(|(head, bodies)| match &bodies[..] {
            [] => unreachable!("Heads always have a body"),
            [body] => body_to_clauses(Inference::new(head.clone()).pos(), body),
            bodies => {
                let mut clauses = vec![];
                bodies
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, body)| {
                        body_to_clauses(InferenceHelper::new(idx, head.clone()).pos(), body)
                    })
                    .collect_into(&mut clauses);
                let helpers: Vec<_> = (0..bodies.len())
                    .map(|idx| InferenceHelper::new(idx, head.clone()).pos())
                    .collect();
                let mut right_implification: Clause = helpers.iter().cloned().collect();
                right_implification.push(Inference::new(head.clone()).neg());
                clauses.push(right_implification);
                helpers
                    .into_iter()
                    .map(|helper| {
                        Clause::from(vec![Inference::new(head.clone()).pos(), helper.negative()])
                    })
                    .collect_into(&mut clauses);
                clauses
            }
        })
}
