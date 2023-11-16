use crate::clauses::Atom;

use self::private::Private;

#[derive(Clone)]
pub enum Literal {
    Pos(String),
    Neg(String),
}

pub trait IntoLiteral: Sized + private::Private {
    fn into_literal(self) -> String;
    fn pos(self) -> Literal {
        Literal::Pos(IntoLiteral::into_literal(self))
    }
    fn neg(self) -> Literal {
        Literal::Neg(IntoLiteral::into_literal(self))
    }
}

mod private {
    pub trait Private {}
}

impl Literal {
    pub fn negative(self) -> Self {
        Self::Neg(self.into_inner())
    }

    pub fn positive(self) -> Self {
        Self::Pos(self.into_inner())
    }

    pub fn into_inner(self) -> String {
        match self {
            Literal::Pos(inner) | Literal::Neg(inner) => inner,
        }
    }
}

pub struct Inference<A: Atom> {
    pub elem: A,
}

impl<A: Atom> Inference<A> {
    pub fn new(elem: A) -> Self {
        Self { elem }
    }
}

pub struct InferenceHelper<A: Atom> {
    pub idx: usize,
    pub head: A,
}
impl<A: Atom> InferenceHelper<A> {
    pub fn new(idx: usize, head: A) -> Self {
        Self { idx, head }
    }
}

pub struct SetInference<A: Atom> {
    pub elem: A,
}
impl<A: Atom> SetInference<A> {
    pub fn new(elem: A) -> Self {
        Self { elem }
    }
}

pub struct SetInferenceHelper<A: Atom> {
    pub idx: usize,
    pub head: A,
}

impl<A: Atom> SetInferenceHelper<A> {
    pub fn new(idx: usize, head: A) -> Self {
        Self { idx, head }
    }
}

pub struct Inverse<A: Atom> {
    pub from: A,
    pub to: A,
}

impl<A: Atom> Inverse<A> {
    pub fn new(from: A, to: A) -> Self {
        Self { from, to }
    }
}

impl<A: Atom> Private for Inference<A> {}
impl<A: Atom> IntoLiteral for Inference<A> {
    fn into_literal(self) -> String {
        let Self { elem } = self;
        format!("inference_{elem}")
    }
}

impl<A: Atom> Private for InferenceHelper<A> {}
impl<A: Atom> IntoLiteral for InferenceHelper<A> {
    fn into_literal(self) -> String {
        let Self { idx, head } = self;
        format!("inference_helper_{idx}_{head}")
    }
}

impl<A: Atom> Private for SetInference<A> {}
impl<A: Atom> IntoLiteral for SetInference<A> {
    fn into_literal(self) -> String {
        let Self { elem } = self;
        format!("set_inference_{elem}")
    }
}

impl<A: Atom> Private for SetInferenceHelper<A> {}
impl<A: Atom> IntoLiteral for SetInferenceHelper<A> {
    fn into_literal(self) -> String {
        let Self { idx, head } = self;
        format!("set_inference_helper_{idx}_{head}")
    }
}

impl<A: Atom> Private for Inverse<A> {}
impl<A: Atom> IntoLiteral for Inverse<A> {
    fn into_literal(self) -> String {
        let Self { from, to } = self;
        format!("inv_{from}_{to}")
    }
}
