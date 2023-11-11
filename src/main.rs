#![feature(iter_collect_into)]
use std::collections::{HashMap, HashSet};

use cadical::Solver;

type Clause = Vec<Literal>;
type RawClause = Vec<i32>;

#[derive(Debug)]
struct Aba {
    rules: Vec<(char, HashSet<char>)>,
    inverses: HashMap<char, char>,
}

enum Literal {
    Pos(String),
    Neg(String),
}

#[derive(Debug)]
struct Mapper {
    map: HashMap<String, u32>,
}

impl Mapper {
    fn new() -> Self {
        Mapper {
            map: HashMap::new(),
        }
    }

    fn as_raw_iter<'s, I: IntoIterator<Item = Clause> + 's>(
        &'s mut self,
        aba_clauses: I,
    ) -> impl Iterator<Item = RawClause> + 's {
        aba_clauses
            .into_iter()
            .map(|clause| clause.iter().map(|lit| self.as_raw(lit)).collect())
    }

    fn as_raw(&mut self, lit: &Literal) -> i32 {
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
}

impl Aba {
    fn new() -> Self {
        Aba {
            rules: vec![],
            inverses: HashMap::new(),
        }
    }

    fn with_assumption(mut self, assumption: char, inverse: char) -> Self {
        self.inverses.insert(assumption, inverse);
        self
    }

    fn with_rule<B: IntoIterator<Item = char>>(mut self, head: char, body: B) -> Self {
        self.rules.push((head, body.into_iter().collect()));
        self
    }

    fn as_clauses(&self) -> Vec<Vec<Literal>> {
        let mut clauses = vec![];
        for (head, body) in &self.rules {
            let mut first = body
                .iter()
                .map(|c| Literal::Neg(format!("support_{c}")))
                .collect::<Vec<_>>();
            first.push(Literal::Pos(format!("support_{head}")));
            clauses.push(first);
            body.iter()
                .map(|c| {
                    vec![
                        Literal::Neg(format!("support_{head}")),
                        Literal::Pos(format!("support_{c}")),
                    ]
                })
                .collect_into(&mut clauses);
        }
        for (assumption, inverses) in &self.inverses {
            clauses.push(vec![Literal::Pos(format!("inv_{assumption}_{inverses}"))]);
        }
        let elements = self
            .inverses
            .keys()
            .chain(self.inverses.values())
            .chain(self.rules.iter().flat_map(|(_, body)| body))
            .chain(self.rules.iter().map(|(head, _)| head))
            .collect::<HashSet<_>>();
        for element in elements {
            for assumption in self.inverses.keys() {
                clauses.push(vec![
                    Literal::Neg(format!("support_{assumption}")),
                    Literal::Neg(format!("support_{element}")),
                    Literal::Neg(format!("inv_{assumption}_{element}")),
                ])
            }
        }
        clauses
    }

    fn clauses_for_assumption_set(&self, assumption_set: Vec<char>) -> Vec<Vec<Literal>> {
        let mut clauses = vec![];
        for assumption in self.inverses.keys() {
            if assumption_set.contains(assumption) {
                clauses.push(vec![Literal::Pos(format!("support_{assumption}"))])
            } else {
                clauses.push(vec![Literal::Neg(format!("support_{assumption}"))])
            }
        }
        clauses
    }
}

fn main() {
    let aba = Aba::new()
        .with_assumption('a', 'r')
        .with_assumption('b', 's')
        .with_assumption('c', 't')
        .with_rule('p', ['q', 'a'])
        .with_rule('q', [])
        .with_rule('r', ['b', 'c']);
    let aba_clauses = aba.as_clauses();

    let extra_clauses = aba.clauses_for_assumption_set(vec!['a', 'b', 'c']);

    let mut map = Mapper::new();
    let mut sat: Solver = Default::default();
    map.as_raw_iter(aba_clauses)
        .for_each(|clause| sat.add_clause(clause));
    map.as_raw_iter(extra_clauses)
        .for_each(|clause| sat.add_clause(clause));

    match sat.solve() {
        Some(true) => {
            println!("True");
        }
        Some(false) => {
            println!("False");
        }
        None => {
            println!("No response");
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

impl std::ops::Deref for Literal {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self {
            Literal::Pos(inner) | Literal::Neg(inner) => inner,
        }
    }
}
