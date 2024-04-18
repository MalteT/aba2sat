use std::fmt::Debug;

pub mod lits {
    use crate::aba::Num;

    macro_rules! into_raw {
        ($ty:ident) => {
            impl From<$ty> for crate::literal::RawLiteral {
                fn from(value: $ty) -> crate::literal::RawLiteral {
                    crate::literal::RawLiteral::$ty(value)
                }
            }
        };
        ($ty:ident from $other:ident) => {
            impl From<$ty> for crate::literal::RawLiteral {
                fn from(value: $ty) -> crate::literal::RawLiteral {
                    crate::literal::RawLiteral::$ty(value)
                }
            }

            impl From<$other> for $ty {
                fn from(value: $other) -> Self {
                    Self(value)
                }
            }
        };
        ($ty:ident from $( $other:ident ),+ ) => {
            impl From<$ty> for crate::literal::RawLiteral {
                fn from(value: $ty) -> crate::literal::RawLiteral {
                    crate::literal::RawLiteral::$ty(value)
                }
            }

            #[allow(non_snake_case)]
            impl From<($( $other ,)+)> for $ty {
                fn from(($( $other ),+): ($( $other ),+)) -> Self {
                    Self($( $other ),+)
                }
            }
        };
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Theory(Num);
    into_raw!(Theory from Num);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TheoryHelper(usize, Num);
    into_raw!(TheoryHelper from usize, Num);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TheoryRuleBodyActive(usize);
    into_raw!(TheoryRuleBodyActive from usize);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TheorySet(Num);
    into_raw!(TheorySet from Num);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TheorySetHelper(usize, Num);
    into_raw!(TheorySetHelper from usize, Num);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TheorySetRuleBodyActive(usize);
    into_raw!(TheorySetRuleBodyActive from usize);

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct LoopHelper(usize);
    into_raw!(LoopHelper from usize);
}

/// A Literal can be used in SAT [`Clause`](crate::clauses::Clause)s
#[derive(Clone)]
pub enum Literal {
    Pos(RawLiteral),
    Neg(RawLiteral),
}

/// All SAT-encodable literals
///
/// This is a single type to ease memory and logic, at the cost of having to
/// extend this type for every new literal type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawLiteral {
    Theory(lits::Theory),
    TheoryHelper(lits::TheoryHelper),
    TheoryRuleBodyActive(lits::TheoryRuleBodyActive),
    TheorySet(lits::TheorySet),
    TheorySetHelper(lits::TheorySetHelper),
    TheorySetRuleBodyActive(lits::TheorySetRuleBodyActive),
    LoopHelper(lits::LoopHelper),
}

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

/// Implement [`IntoLiteral`] for all types that can be converted into [`RawLiteral`]s.
impl<T: Into<RawLiteral>> IntoLiteral for T {
    fn into_literal(self) -> RawLiteral {
        self.into()
    }
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
    type Target = RawLiteral;

    fn deref(&self) -> &Self::Target {
        match self {
            Literal::Pos(inner) | Literal::Neg(inner) => inner,
        }
    }
}

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Pos(inner) => write!(f, "+{inner:?}"),
            Literal::Neg(inner) => write!(f, "-{inner:?}"),
        }
    }
}
