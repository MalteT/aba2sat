#![feature(iter_collect_into)]
use std::collections::{HashMap, HashSet};

use cadical::Solver;
use clauses::{Clause, ClauseList, Literal};
use mapper::Mapper;

mod clauses;
mod mapper;

#[derive(Debug)]
struct Aba {
    rules: Vec<(char, HashSet<char>)>,
    inverses: HashMap<char, char>,
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

    fn as_clauses(&self) -> ClauseList {
        let mut clauses = ClauseList::new();
        for (head, body) in &self.rules {
            let mut first = body
                .iter()
                .map(|c| Literal::Neg(format!("support_{c}")))
                .collect::<Clause>();
            first.push(Literal::Pos(format!("support_{head}")));
            clauses.push(first);
            body.iter()
                .map(|c| {
                    vec![
                        Literal::Neg(format!("support_{head}")),
                        Literal::Pos(format!("support_{c}")),
                    ]
                    .into()
                })
                .collect_into(&mut clauses);
        }
        for (assumption, inverses) in &self.inverses {
            clauses.push(vec![Literal::Pos(format!("inv_{assumption}_{inverses}"))].into());
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
                clauses.push(
                    vec![
                        Literal::Neg(format!("support_{assumption}")),
                        Literal::Neg(format!("support_{element}")),
                        Literal::Neg(format!("inv_{assumption}_{element}")),
                    ]
                    .into(),
                )
            }
        }
        clauses
    }

    fn clauses_for_assumption_set(&self, assumption_set: Vec<char>) -> ClauseList {
        let mut clauses = vec![];
        for assumption in self.inverses.keys() {
            if assumption_set.contains(assumption) {
                clauses.push(vec![Literal::Pos(format!("support_{assumption}"))].into())
            } else {
                clauses.push(vec![Literal::Neg(format!("support_{assumption}"))].into())
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
