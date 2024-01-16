//! # Assumption-based Argumentation
//!
//! All relevant tools for solving problems around assumption-based argumentation.
//!
//! ## Example
//! ```
//! # use aba2sat::aba::Aba;
//! # use aba2sat::aba::problems::solve;
//! # use aba2sat::aba::problems::admissibility::VerifyAdmissibleExtension;
//! let aba =
//!     // Start with an empty framework
//!     Aba::default()
//!         // Add an assumption 'a' with inverse 'p'
//!         .with_assumption('a', 'p')
//!         // Add an assumption 'b' with inverse 'q'
//!         .with_assumption('b', 'q')
//!         // Add a rule to derive 'p' (the inverse of 'a') from 'b'
//!         .with_rule('p', ['b']);
//!
//!
//! // Solve the problem whether the set of assumptions {'b'} is admissible
//! let result =
//!     solve(VerifyAdmissibleExtension { assumptions: vec!['b'] }, &aba).unwrap();
//!
//! // The result should be true
//! assert!(result)
//! ```
use std::collections::{HashMap, HashSet};

use crate::{
    clauses::{Atom, Clause, ClauseList},
    literal::{IntoLiteral, Literal, TheoryAtom},
};

pub mod problems;

pub type Rule<A> = (A, HashSet<A>);
pub type RuleList<A> = Vec<Rule<A>>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Aba<A: Atom> {
    pub rules: RuleList<A>,
    pub inverses: HashMap<A, A>,
}

#[derive(Debug)]
pub struct Theory<A: Atom>(A);

#[derive(Debug)]
pub struct TheoryHelper<A: Atom>(usize, A);

impl<A: Atom> Aba<A> {
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

    /// Translate the ABA into base rules / definitions for SAT solving
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

    /// Filtered list of rules
    ///
    /// Iterates over all rules, marking reachable elements until
    /// no additional rule can be applied. Then removes every
    /// rule that contains any unreachable atom and returns the rest
    pub fn trim(&mut self) {
        // Begin with all assumptions marked as reachable
        let mut reachable: HashSet<_> = self.assumptions().cloned().collect();
        // Calculate all reachable elements
        loop {
            let mut marked_any = false;
            for (head, body) in &self.rules {
                if reachable.contains(head) {
                    continue;
                }
                if body.iter().all(|atom| reachable.contains(atom)) {
                    marked_any = true;
                    reachable.insert(head.clone());
                }
            }
            if !marked_any {
                break;
            }
        }
        // Remove all rules that contain any unreachable atom
        self.rules.retain(|(head, body)| {
            // Both the head and all elements from the body must be reachable
            reachable.contains(head) && body.iter().all(|atom| reachable.contains(atom))
        });
    }

    fn derive_rule_clauses(&self) -> impl Iterator<Item = Clause> + '_ {
        theory_helper::<Theory<_>, _>(self)
    }

    fn rule_heads(&self) -> impl Iterator<Item = &A> + '_ {
        self.rules.iter().map(|(head, _)| head)
    }

    fn has_assumption(&self, atom: &A) -> bool {
        self.inverses.contains_key(atom)
    }

    fn has_element(&self, element: &A) -> bool {
        self.universe().any(|e| element == e)
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

/// Generate the logic for theory derivation in the given [`Aba`]
///
/// This will need a valid [`TheoryAtom`] that will be used to construct the logic
// TODO: describe how this is done
pub fn theory_helper<I: TheoryAtom<A> + IntoLiteral, A: Atom>(
    aba: &Aba<A>,
) -> impl Iterator<Item = Clause> + '_ {
    // The combined list of rules, such that every
    // head is unique and possible contains a list of bodies
    let mut rules_combined =
        aba.rules
            .iter()
            .fold(HashMap::<_, Vec<_>>::new(), |mut rules, (head, body)| {
                rules.entry(head).or_default().push(body);
                rules
            });
    // All atoms that can be derived by rules
    let rule_heads: HashSet<_> = aba.rule_heads().collect();
    // For every non-assumption, that is not derivable add a rule without a body,
    // such that it cannot be derived at all. This is to prevent the solver from
    // guessing this atom on it's own
    aba.universe()
        .filter(|atom| !aba.has_assumption(atom))
        .filter(|atom| !rule_heads.contains(atom))
        .map(|atom| (atom, vec![]))
        .collect_into(&mut rules_combined);
    // All combined rules
    // These are heads with any number of bodies, possibly none
    rules_combined
        .into_iter()
        .flat_map(|(head, bodies)| match &bodies[..] {
            // No bodies, add a clause that prevents the head from accuring in the theory
            [] => {
                vec![Clause::from(vec![I::new(head.clone()).neg()])]
            }
            // A single body only, this is equivalent to a head that can only be derived by a single rule
            [body] => body_to_clauses::<I, _>(I::new(head.clone()).pos(), body),
            // n bodies, we'll need to take extra care to allow any number of bodies to derive this
            // head without logic errors
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
