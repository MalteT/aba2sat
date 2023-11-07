#![feature(iter_collect_into)]
use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use cadical::Solver;

#[derive(Debug)]
struct Aba {
    rules: Vec<(char, HashSet<char>)>,
    inverses: HashMap<char, char>,
}

enum Literal {
    Pos(String),
    Neg(String),
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

    fn clauses_for(&self, assumption_set: Vec<char>) -> Vec<Vec<Literal>> {
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

fn map_clauses(
    aba_clauses: Vec<Vec<Literal>>,
    mappings: &mut Vec<String>,
) -> impl Iterator<Item = Vec<i32>> + '_ {
    aba_clauses.into_iter().scan(mappings, |map, clause| {
        let clause = clause
            .into_iter()
            .map(|lit| {
                let existing_mapping = map
                    .iter()
                    .enumerate()
                    .find(|(_, item)| *item == &*lit)
                    .map(|(idx, _)| idx as i32 + 1);
                match existing_mapping {
                    Some(idx) => match lit {
                        Literal::Pos(_) => idx,
                        Literal::Neg(_) => -idx,
                    },
                    None => {
                        map.push(lit.deref().clone());
                        let idx = map.len() as i32;
                        match lit {
                            Literal::Pos(_) => idx,
                            Literal::Neg(_) => -idx,
                        }
                    }
                }
            })
            .collect();
        Some(clause)
    })
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

    let extra_clauses = aba.clauses_for(vec!['a', 'b', 'c']);

    let mut mappings = vec![];
    let mut sat: Solver = Default::default();
    map_clauses(aba_clauses, &mut mappings).for_each(|clause| sat.add_clause(clause));
    map_clauses(extra_clauses, &mut mappings).for_each(|clause| sat.add_clause(clause));
    println!("{:?}", sat.solve());
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
