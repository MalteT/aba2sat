use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Deref, DerefMut},
};

use crate::literal::Literal;

pub type ClauseList = Vec<Clause>;
pub type RawClause = Vec<RawLiteral>;
pub type RawLiteral = i32;

/// Generic Atom that can be used to construct [`Clause`]s.
#[doc(notable_trait)]
pub trait Atom: Debug + Display + Hash + Eq + Clone + 'static {}

impl<A: Debug + Display + Hash + Eq + Clone + 'static> Atom for A {}

/// A disjunction of [`Literal`]s.
pub struct Clause {
    list: Vec<Literal>,
}

impl std::fmt::Debug for Clause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let list: String = self
            .iter()
            .map(|lit| format!("{:?}", lit))
            .intersperse(String::from(" "))
            .collect();
        write!(f, "{list}}}")
    }
}

impl Deref for Clause {
    type Target = Vec<Literal>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for Clause {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl FromIterator<Literal> for Clause {
    fn from_iter<T: IntoIterator<Item = Literal>>(iter: T) -> Self {
        Clause {
            list: Vec::from_iter(iter),
        }
    }
}

impl From<Vec<Literal>> for Clause {
    fn from(list: Vec<Literal>) -> Self {
        Clause { list }
    }
}
