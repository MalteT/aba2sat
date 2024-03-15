#![cfg(test)]

use std::collections::{BTreeMap, HashMap, HashSet};

use super::{Aba, Num};

#[derive(Debug, Clone)]
pub struct DebugAba {
    aba: Aba,
    forward_map: HashMap<char, Num>,
    backward_map: BTreeMap<Num, char>,
    next: Num,
}

impl DebugAba {
    pub fn with_assumption(mut self, assumption: char, inverse: char) -> Self {
        let assumption = self.forward(assumption);
        let inverse = self.forward(inverse);
        self.aba = self.aba.with_assumption(assumption, inverse);
        self
    }

    pub fn with_rule<'n, B: IntoIterator<Item = char>>(mut self, head: char, body: B) -> Self {
        let head = self.forward(head);
        let body: Vec<_> = body
            .into_iter()
            .scan(&mut self, |aba, elem| {
                let elem = aba.forward(elem);
                Some(elem)
            })
            .collect();
        self.aba = self.aba.with_rule(head, body);
        self
    }

    pub fn aba(&self) -> &Aba {
        &self.aba
    }

    pub fn forward_atom(&self, atom: char) -> Option<Num> {
        self.forward_map.get(&atom).cloned()
    }

    pub fn forward_set(&self, set: HashSet<char>) -> Option<HashSet<Num>> {
        set.into_iter()
            .map(|atom| self.forward_atom(atom))
            .collect()
    }

    pub fn forward_sets<S: IntoIterator<Item = HashSet<char>>>(
        &self,
        sets: S,
    ) -> Option<Vec<HashSet<Num>>> {
        sets.into_iter().map(|set| self.forward_set(set)).collect()
    }

    pub fn backward_sets<S: IntoIterator<Item = HashSet<Num>>>(
        &self,
        sets: S,
    ) -> Option<Vec<HashSet<char>>> {
        sets.into_iter()
            .map(|set| {
                set.into_iter()
                    .map(|atom| self.backward_map.get(&atom).cloned())
                    .collect()
            })
            .collect()
    }

    fn forward(&mut self, atom: char) -> Num {
        match self.forward_map.get(&atom) {
            Some(atom) => *atom,
            None => {
                let next = self.next;
                self.next += 1;
                self.forward_map.insert(atom, next);
                self.backward_map.insert(next, atom);
                next
            }
        }
    }
}

impl Default for DebugAba {
    fn default() -> Self {
        Self {
            aba: Default::default(),
            forward_map: Default::default(),
            backward_map: Default::default(),
            next: 1,
        }
    }
}
