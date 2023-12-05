use crate::clauses::Atom;

mod private {
    pub trait Private {}
}

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

pub trait InferenceAtom<A: Atom>: Sized + private::Private + IntoLiteral {
    type Helper: InferenceAtomHelper<A>;
    fn new(atom: A) -> Self;
}

pub trait InferenceAtomHelper<A: Atom>: Sized + private::Private + IntoLiteral {
    fn new(idx: usize, atom: A) -> Self;
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

// TODO: Let the problems define these things
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

// TODO: Let the problems define these things
pub struct OpponentInference<A: Atom> {
    pub elem: A,
}

impl<A: Atom> OpponentInference<A> {
    pub fn new(elem: A) -> Self {
        Self { elem }
    }
}

pub struct OpponentInferenceHelper<A: Atom> {
    pub idx: usize,
    pub head: A,
}

impl<A: Atom> OpponentInferenceHelper<A> {
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

pub struct SetAttack<A: Atom> {
    pub against: A,
}

impl<A: Atom> SetAttack<A> {
    pub fn new(against: A) -> Self {
        Self { against }
    }
}

impl<A: Atom> Private for Inference<A> {}
impl<A: Atom> IntoLiteral for Inference<A> {
    fn into_literal(self) -> String {
        let Self { elem } = self;
        format!("inference_{elem}")
    }
}
impl<A: Atom> InferenceAtom<A> for Inference<A> {
    type Helper = InferenceHelper<A>;

    fn new(atom: A) -> Self {
        Self::new(atom)
    }
}

impl<A: Atom> Private for InferenceHelper<A> {}
impl<A: Atom> IntoLiteral for InferenceHelper<A> {
    fn into_literal(self) -> String {
        let Self { idx, head } = self;
        format!("inference_helper_{idx}_{head}")
    }
}
impl<A: Atom> InferenceAtomHelper<A> for InferenceHelper<A> {
    fn new(idx: usize, atom: A) -> Self {
        Self::new(idx, atom)
    }
}

impl<A: Atom> Private for SetInference<A> {}
impl<A: Atom> IntoLiteral for SetInference<A> {
    fn into_literal(self) -> String {
        let Self { elem } = self;
        format!("set_inference_{elem}")
    }
}
impl<A: Atom> InferenceAtom<A> for SetInference<A> {
    type Helper = SetInferenceHelper<A>;

    fn new(atom: A) -> Self {
        Self::new(atom)
    }
}

impl<A: Atom> Private for SetInferenceHelper<A> {}
impl<A: Atom> IntoLiteral for SetInferenceHelper<A> {
    fn into_literal(self) -> String {
        let Self { idx, head } = self;
        format!("set_inference_helper_{idx}_{head}")
    }
}
impl<A: Atom> InferenceAtomHelper<A> for SetInferenceHelper<A> {
    fn new(idx: usize, atom: A) -> Self {
        Self::new(idx, atom)
    }
}

impl<A: Atom> Private for OpponentInference<A> {}
impl<A: Atom> IntoLiteral for OpponentInference<A> {
    fn into_literal(self) -> String {
        let Self { elem } = self;
        format!("set_inference_{elem}")
    }
}
impl<A: Atom> InferenceAtom<A> for OpponentInference<A> {
    type Helper = OpponentInferenceHelper<A>;

    fn new(atom: A) -> Self {
        Self::new(atom)
    }
}

impl<A: Atom> Private for OpponentInferenceHelper<A> {}
impl<A: Atom> IntoLiteral for OpponentInferenceHelper<A> {
    fn into_literal(self) -> String {
        let Self { idx, head } = self;
        format!("set_inference_helper_{idx}_{head}")
    }
}
impl<A: Atom> InferenceAtomHelper<A> for OpponentInferenceHelper<A> {
    fn new(idx: usize, atom: A) -> Self {
        Self::new(idx, atom)
    }
}

impl<A: Atom> Private for Inverse<A> {}
impl<A: Atom> IntoLiteral for Inverse<A> {
    fn into_literal(self) -> String {
        let Self { from, to } = self;
        format!("inv_{from}_{to}")
    }
}

impl<A: Atom> Private for SetAttack<A> {}
impl<A: Atom> IntoLiteral for SetAttack<A> {
    fn into_literal(self) -> String {
        let Self { against } = self;
        format!("set_attack_{against}")
    }
}
