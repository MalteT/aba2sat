use std::ops::{Deref, DerefMut};

use crate::literal::Literal;

pub type ClauseList = Vec<Clause>;
pub type RawClause = Vec<RawLiteral>;
pub type RawLiteral = i32;
pub type Atom = u32;

pub struct Clause {
    list: Vec<Literal>,
}

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Pos(str) => write!(f, "+{str}"),
            Literal::Neg(str) => write!(f, "-{str}"),
        }
    }
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

impl std::ops::Deref for Literal {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self {
            Literal::Pos(inner) | Literal::Neg(inner) => inner,
        }
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
