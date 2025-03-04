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
//! let atom = aba.forward_atom('b').unwrap();
//! let assumptions = vec![atom].into_iter().collect();
//! let result =
//!     solve(VerifyAdmissibleExtension { assumptions }, aba.aba().clone(), None).unwrap();
//!
//! // The result should be true
//! assert!(result)
//! ```
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::literal::RawLiteral;

pub mod debug;
mod prepared;
pub mod problems;
mod theory;
mod traverse;

pub use prepared::PreparedAba;
pub use traverse::{Loop, Loops};

pub type Rule = (Num, BTreeSet<Num>);
pub type RuleList = Vec<Rule>;
pub type Num = u32;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Aba {
    pub rules: RuleList,
    pub inverses: HashMap<Num, Num>,
}

impl Aba {
    pub fn with_assumption(mut self, assumption: Num, inverse: Num) -> Self {
        self.inverses.insert(assumption, inverse);
        self
    }

    pub fn with_rule<B: IntoIterator<Item = Num>>(mut self, head: Num, body: B) -> Self {
        let mut body_trans = BTreeSet::new();
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
    #[cfg_attr(
        feature = "timing",
        fun_time::fun_time(
            message = "Preparing ABA with max {max_loops:?} loops",
            reporting = "log"
        )
    )]
    pub fn prepare(self, max_loops: Option<usize>) -> PreparedAba {
        PreparedAba::new(self, max_loops)
    }

    fn rule_heads(&self) -> impl Iterator<Item = &Num> + '_ {
        self.rules.iter().map(|(head, _)| head)
    }
}

pub trait Context {
    type Base: From<Num> + Into<RawLiteral> + 'static;
    type Rule: From<usize> + Into<RawLiteral> + 'static;
    type Loop: From<usize> + Into<RawLiteral> + 'static;
}

impl Context for crate::literal::lits::Candidate {
    type Base = Self;
    type Rule = crate::literal::lits::CandidateRuleBodyActive;
    type Loop = crate::literal::lits::CandidateLoopHelper;
}

impl Context for crate::literal::lits::Attacker {
    type Base = Self;
    type Rule = crate::literal::lits::AttackerRuleBodyActive;
    type Loop = crate::literal::lits::AttackerLoopHelper;
}
