use std::collections::HashMap;

use crate::{
    clauses::{Clause, RawClause},
    literal::{Literal, RawLiteral},
};

#[derive(Debug, Default)]
pub struct Mapper {
    map: HashMap<RawLiteral, u32>,
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
        let key = self.map.get(lit).copied().unwrap_or_else(|| {
            debug_assert!(self.map.len() <= i32::MAX as usize, "Mapper overflowed");
            let new = self.map.len() as u32 + 1;
            self.map.insert(**lit, new);
            new
        }) as i32;
        match lit {
            Literal::Pos(_) => key,
            Literal::Neg(_) => -key,
        }
    }

    #[cfg(debug_assertions)]
    pub fn reconstruct<'s>(
        &'s self,
        sat: &'s cadical::Solver,
    ) -> impl Iterator<Item = Literal> + 's {
        self.map.iter().flat_map(|(lit, raw)| {
            sat.value(*raw as i32).map(|result| match result {
                true => Literal::Pos(*lit),
                false => Literal::Neg(*lit),
            })
        })
    }

    pub fn get_raw(&self, lit: &Literal) -> Option<i32> {
        self.map.get(lit).map(|&raw| raw as i32)
    }
}
