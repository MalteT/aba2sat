use std::collections::{HashMap, HashSet};

use crate::{
    clauses::{Atom, Clause, ClauseList},
    literal::{IntoLiteral, Literal, TheoryAtom},
};

pub mod problems;

#[derive(Debug, Default, PartialEq)]
pub struct Aba<A: Atom> {
    pub rules: Vec<(A, HashSet<A>)>,
    pub inverses: HashMap<A, A>,
}

#[derive(Debug)]
pub struct Theory<A: Atom>(A);

#[derive(Debug)]
pub struct TheoryHelper<A: Atom>(usize, A);

impl<A: Atom> Aba<A> {
    #[cfg(test)]
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

    pub fn assumptions(&self) -> impl Iterator<Item = &A> {
        self.inverses.keys()
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
        inference_helper::<Theory<_>, _>(self)
    }

    fn rule_heads(&self) -> impl Iterator<Item = &A> + '_ {
        self.rules.iter().map(|(head, _)| head)
    }

    fn has_assumption(&self, atom: &A) -> bool {
        self.inverses.contains_key(atom)
    }
}

fn body_to_clauses<I: TheoryAtom<A>, A: Atom>(head: Literal, body: &HashSet<A>) -> ClauseList {
    let mut clauses = vec![];
    let mut left_implication: Clause = body.iter().map(|elem| I::new(elem.clone()).neg()).collect();
    left_implication.push(head.clone().positive());
    clauses.push(left_implication);
    body.iter()
        .map(|elem| vec![head.clone().negative(), I::new(elem.clone()).pos()].into())
        .collect_into(&mut clauses);
    clauses
}

pub fn inference_helper<I: TheoryAtom<A> + IntoLiteral, A: Atom>(
    aba: &Aba<A>,
) -> impl Iterator<Item = Clause> + '_ {
    let mut rules_combined =
        aba.rules
            .iter()
            .fold(HashMap::<_, Vec<_>>::new(), |mut rules, (head, body)| {
                rules.entry(head).or_default().push(body);
                rules
            });
    let rule_heads: HashSet<_> = aba.rule_heads().collect();
    // For every non-assumption, that is not derivable add a rule without a body
    aba.universe()
        .filter(|atom| !aba.has_assumption(atom))
        .filter(|atom| !rule_heads.contains(atom))
        .map(|atom| (atom, vec![]))
        .collect_into(&mut rules_combined);
    rules_combined
        .into_iter()
        .flat_map(|(head, bodies)| match &bodies[..] {
            [] => {
                vec![Clause::from(vec![I::new(head.clone()).neg()])]
            }
            [body] => body_to_clauses::<I, _>(I::new(head.clone()).pos(), body),
            bodies => {
                let mut clauses = vec![];
                bodies
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, body)| {
                        body_to_clauses::<I, _>(I::new_helper(idx, head.clone()).pos(), body)
                    })
                    .collect_into(&mut clauses);
                let helpers: Vec<_> = (0..bodies.len())
                    .map(|idx| I::new_helper(idx, head.clone()).pos())
                    .collect();
                let mut right_implification: Clause = helpers.iter().cloned().collect();
                right_implification.push(I::new(head.clone()).neg());
                clauses.push(right_implification);
                helpers
                    .into_iter()
                    .map(|helper| Clause::from(vec![I::new(head.clone()).pos(), helper.negative()]))
                    .collect_into(&mut clauses);
                clauses
            }
        })
}

impl<A: Atom> TheoryAtom<A> for Theory<A> {
    type Helper = TheoryHelper<A>;

    fn new(atom: A) -> Self {
        Self(atom)
    }

    fn new_helper(idx: usize, atom: A) -> Self::Helper {
        TheoryHelper(idx, atom)
    }
}
