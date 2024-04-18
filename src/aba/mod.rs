//! # Assumption-based Argumentation
//!
//! All relevant tools for solving problems around assumption-based argumentation.
//!
//! ## Example
//! ```
//! # use aba2sat::aba::debug::DebugAba;
//! # use aba2sat::aba::problems::solve;
//! # use aba2sat::aba::problems::admissibility::VerifyAdmissibleExtension;
//! let aba =
//!     // Start with an empty framework
//!     DebugAba::default()
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

use crate::literal::RawLiteral;

use self::prepared::PreparedAba;

pub mod debug;
mod prepared;
pub mod problems;
mod theory;

pub type Rule = (Num, HashSet<Num>);
pub type RuleList = Vec<Rule>;
pub type Num = u32;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Aba {
    rules: RuleList,
    inverses: HashMap<Num, Num>,
}

impl Aba {
    pub fn with_assumption(mut self, assumption: Num, inverse: Num) -> Self {
        self.inverses.insert(assumption, inverse);
        self
    }

    pub fn with_rule<B: IntoIterator<Item = Num>>(mut self, head: Num, body: B) -> Self {
        let mut body_trans = HashSet::new();
        body.into_iter().for_each(|elem| {
            body_trans.insert(elem);
        });
        self.rules.push((head, body_trans));
        self
    }

    pub fn universe(&self) -> impl Iterator<Item = &Num> {
        // List of all elements of our ABA, basically our L (universe)
        self.inverses
            .keys()
            .chain(self.inverses.values())
            .chain(self.rules.iter().flat_map(|(_, body)| body))
            .chain(self.rules.iter().map(|(head, _)| head))
    }

    pub fn assumptions(&self) -> impl Iterator<Item = &Num> {
        self.inverses.keys()
    }

    pub fn contains_assumption(&self, a: &Num) -> bool {
        self.inverses.contains_key(a)
    }

    pub fn contains_atom(&self, elem: &Num) -> bool {
        self.universe().any(|e| *e == *elem)
    }

    #[cfg(debug_assertions)]
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

    /// Prepare this aba for translation to SAT
    pub fn prepare(self) -> PreparedAba {
        PreparedAba::from(self)
    }

    fn rule_heads(&self) -> impl Iterator<Item = &Num> + '_ {
        self.rules.iter().map(|(head, _)| head)
    }
}

pub trait Context {
    type Base: From<Num> + Into<RawLiteral> + 'static;
    type Rule: From<usize> + Into<RawLiteral> + 'static;
}

impl Context for crate::literal::lits::Theory {
    type Base = Self;
    type Rule = crate::literal::lits::TheoryRuleBodyActive;
}

impl Context for crate::literal::lits::TheorySet {
    type Base = Self;
    type Rule = crate::literal::lits::TheorySetRuleBodyActive;
}
