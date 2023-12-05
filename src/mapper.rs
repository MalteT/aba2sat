use std::collections::HashMap;

use cadical::Solver;

use crate::{
    clauses::{Clause, RawClause},
    literal::Literal,
};

#[derive(Debug, Default)]
pub struct Mapper {
    map: HashMap<String, u32>,
}

pub enum ReconstructedLiteral {
    Pos(String),
    Neg(String),
}

impl Mapper {
    pub fn new() -> Self {
        Mapper {
            map: HashMap::new(),
        }
    }

    pub fn as_raw_iter<'s, I: IntoIterator<Item = &'s Clause> + 's>(
        &'s mut self,
        aba_clauses: I,
    ) -> impl Iterator<Item = RawClause> + 's {
        aba_clauses
            .into_iter()
            .map(|clause| clause.iter().map(|lit| self.as_raw(lit)).collect())
    }

    pub fn as_raw(&mut self, lit: &Literal) -> i32 {
        let key = self.map.get(lit.as_str()).copied().unwrap_or_else(|| {
            debug_assert!(self.map.len() <= i32::MAX as usize, "Mapper overflowed");
            let new = self.map.len() as u32 + 1;
            self.map.insert(lit.to_string(), new);
            new
        }) as i32;
        match lit {
            Literal::Pos(_) => key,
            Literal::Neg(_) => -key,
        }
    }

    pub fn reconstruct<'s>(
        &'s self,
        sat: &'s Solver,
    ) -> impl Iterator<Item = ReconstructedLiteral> + 's {
        self.map.iter().flat_map(|(lit, raw)| {
            let (_, lit) = lit.split_once('#').expect("All literals must contain a #");
            sat.value(*raw as i32).map(|result| match result {
                true => ReconstructedLiteral::Pos(lit.to_owned()),
                false => ReconstructedLiteral::Neg(lit.to_owned()),
            })
        })
    }

    pub fn get_raw(&self, lit: &Literal) -> Option<i32> {
        self.map.get(lit.as_str()).map(|&raw| raw as i32)
    }
}

impl std::fmt::Debug for ReconstructedLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReconstructedLiteral::Pos(str) => write!(f, "+{str}"),
            ReconstructedLiteral::Neg(str) => write!(f, "-{str}"),
        }
    }
}
