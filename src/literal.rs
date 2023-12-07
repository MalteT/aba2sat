use std::{
    any::{Any, TypeId},
    fmt::Debug,
};

use crate::clauses::Atom;

/// A Literal can be used in SAT [`Clause`](crate::clauses::Clause)s
#[derive(Clone)]
pub enum Literal {
    Pos(RawLiteral),
    Neg(RawLiteral),
}

/// New type to prevent creation of arbitrary SAT literals
#[derive(Clone, Debug)]
pub struct RawLiteral(String);

/// Convert the type into it's literal
#[doc(notable_trait)]
pub trait IntoLiteral: Sized {
    /// Actual transformation
    fn into_literal(self) -> RawLiteral;
    /// Create a positive literal from this value
    fn pos(self) -> Literal {
        Literal::Pos(IntoLiteral::into_literal(self))
    }
    /// Create a negative literal from this value
    fn neg(self) -> Literal {
        Literal::Neg(IntoLiteral::into_literal(self))
    }
}

/// Implement [`IntoLiteral`] for all types that are 'static, sized and debuggable
impl<T: Any + Debug + Sized> IntoLiteral for T {
    fn into_literal(self) -> RawLiteral {
        RawLiteral(format!("{:?}#{:?}", TypeId::of::<T>(), self))
    }
}

/// ([`Literal`]) A literal that can be used to construct the logic behind
/// the theory of a (sub-)set of assumptions (`th(S)`) in an [`Aba`](crate::aba::Aba).
///
/// See [`crate::aba::inference_helper`].
pub trait TheoryAtom<A: Atom>: Sized + IntoLiteral {
    /// Helper type
    type Helper: IntoLiteral;
    /// Construct this [`Literal`]
    fn new(atom: A) -> Self;
    /// Construct the helper [`Literal`]
    fn new_helper(idx: usize, atom: A) -> Self::Helper;
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
