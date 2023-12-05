use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

use crate::clauses::Atom;

#[derive(Clone)]
pub enum Literal {
    Pos(RawLiteral),
    Neg(RawLiteral),
}

#[derive(Clone, Debug)]
pub struct RawLiteral(String);

pub trait IntoLiteral: Sized {
    fn into_literal(self) -> RawLiteral;
    fn pos(self) -> Literal {
        Literal::Pos(IntoLiteral::into_literal(self))
    }
    fn neg(self) -> Literal {
        Literal::Neg(IntoLiteral::into_literal(self))
    }
}

impl<T: Any + Debug + Sized> IntoLiteral for T {
    fn into_literal(self) -> RawLiteral {
        RawLiteral(format!("{:?}#{:?}", TypeId::of::<T>(), self))
    }
}

pub trait InferenceAtom<A: Atom>: Sized + IntoLiteral {
    type Helper: InferenceAtomHelper<A>;
    fn new(atom: A) -> Self;
}

pub trait InferenceAtomHelper<A: Atom>: Sized + IntoLiteral {
    fn new(idx: usize, atom: A) -> Self;
}

impl Literal {
    pub fn negative(self) -> Self {
        Self::Neg(self.into_inner())
    }

    pub fn positive(self) -> Self {
        Self::Pos(self.into_inner())
    }

    pub fn into_inner(self) -> RawLiteral {
        match self {
            Literal::Pos(inner) | Literal::Neg(inner) => inner,
        }
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

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Pos(str) => write!(f, "+{str}"),
            Literal::Neg(str) => write!(f, "-{str}"),
        }
    }
}

impl std::ops::Deref for RawLiteral {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for RawLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
