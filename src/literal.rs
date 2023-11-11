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

    pub fn positive(self) -> Literal {
        Self::Pos(self.into_inner())
    }

    pub fn into_inner(self) -> String {
        match self {
            Literal::Pos(inner) | Literal::Neg(inner) => inner,
        }
    }
}

pub struct Inference {
    pub elem: char,
}

pub struct InferenceHelper {
    pub idx: usize,
    pub head: char,
}

pub struct SetInference {
    pub elem: char,
}

pub struct SetInferenceHelper {
    pub idx: usize,
    pub head: char,
}

pub struct Inverse {
    pub from: char,
    pub to: char,
}

impl Private for Inference {}
impl IntoLiteral for Inference {
    fn into_literal(self) -> String {
        let Self { elem } = self;
        format!("inference_{elem}")
    }
}

impl Private for InferenceHelper {}
impl IntoLiteral for InferenceHelper {
    fn into_literal(self) -> String {
        let Self { idx, head } = self;
        format!("inference_helper_{idx}_{head}")
    }
}

impl Private for SetInference {}
impl IntoLiteral for SetInference {
    fn into_literal(self) -> String {
        let Self { elem } = self;
        format!("set_inference_{elem}")
    }
}

impl Private for SetInferenceHelper {}
impl IntoLiteral for SetInferenceHelper {
    fn into_literal(self) -> String {
        let Self { idx, head } = self;
        format!("set_inference_helper_{idx}_{head}")
    }
}

impl Private for Inverse {}
impl IntoLiteral for Inverse {
    fn into_literal(self) -> String {
        let Self { from, to } = self;
        format!("inv_{from}_{to}")
    }
}
