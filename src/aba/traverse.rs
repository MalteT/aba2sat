use std::collections::HashSet;

use super::{Aba, Num};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
struct RuleIdx(usize);

impl RuleIdx {
    fn advance(&mut self) {
        self.0 += 1;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
struct UniverseIdx(usize);

#[derive(Debug)]
pub struct Loops<'a> {
    aba: &'a Aba,
    active: HashSet<Num>,
    rule_indices: Vec<RuleIdx>,
    found: Vec<Loop>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loop {
    pub heads: HashSet<Num>,
}

pub fn loops_of(aba: &Aba) -> Loops<'_> {
    let active = aba.assumptions().cloned().collect();
    Loops {
        aba,
        active,
        rule_indices: vec![RuleIdx(0)],
        found: vec![],
    }
}

impl<'a> Iterator for Loops<'a> {
    type Item = Loop;

    // TODO: We could run Tarjan first to split everything into SCCs to decrease the exponent of the runtime of this algorithm with linear overhead
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Ensure that the rule_indices list is not empty, if it is, we're done
            if self.rule_indices.is_empty() {
                break None;
            }
            // Safe! We've exited already, if rule_indices is empty
            let rule_idx = self.rule_indices.last().unwrap();
            // Ensure that the rule_idx is valid, if it is not, backtrack
            if rule_idx.0 >= self.aba.rules.len() {
                // We're at the end of our rule list, backtrack
                self.rule_indices.pop();
                match self.rule_indices.last_mut() {
                    Some(idx) => {
                        let (head, _body) = &self.aba.rules[idx.0];
                        self.active.remove(head);
                        idx.advance();
                        continue;
                    }
                    None => {
                        // We popped the last rule, iterator ends here
                        break None;
                    }
                }
            }
            // Ensure that the rule_idx does not point to a rule that has been applied already
            if self.rule_indices[0..self.rule_indices.len() - 1].contains(rule_idx) {
                // The rule was applied before
                self.rule_indices.last_mut().unwrap().advance();
                continue;
            }
            // Ensure causality
            if self.rule_indices.len() >= 2 {
                // There was a rule before this one, ensure that the causality works
                // i.e. the next rule should contain the head of the last rule
                let last_rule_idx = self.rule_indices[self.rule_indices.len() - 2];
                let (last_rule_head, _) = self.aba.rules[last_rule_idx.0];
                if !self.aba.rules[rule_idx.0].1.contains(&last_rule_head) {
                    self.rule_indices.last_mut().unwrap().advance();
                    continue;
                }
            }
            // Still rules to try
            let (head, body) = &self.aba.rules[rule_idx.0];
            if body.is_subset(&self.active) {
                // The rule can be applied
                if self.active.contains(head) {
                    // The rule head is already active, loop found!
                    let heads = self.rule_indices[0..self.rule_indices.len() - 1]
                        .iter()
                        .rev()
                        .map_while(|rule_idx| {
                            let (old_head, _body) = &self.aba.rules[rule_idx.0];
                            if *old_head != *head {
                                Some(*old_head)
                            } else {
                                None
                            }
                        })
                        // Add the current head to the loop, as it is not in the active list
                        .chain(std::iter::once(*head))
                        .collect();
                    // Advance the rule index for the next call
                    // Safe! No adjustment to the rule_indices since the last check at the start of the loop
                    self.rule_indices.last_mut().unwrap().advance();
                    let new_loop = Loop { heads };
                    if !self.found.contains(&new_loop) {
                        self.found.push(new_loop.clone());
                        break Some(new_loop);
                    } else {
                        continue;
                    }
                }
                // The rule could be applied and does not cause a conflict
                // Go one level deeper
                self.rule_indices.push(RuleIdx(0));
                self.active.insert(*head);
            } else {
                self.rule_indices.last_mut().unwrap().advance();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::aba::debug::DebugAba;

    use super::*;

    #[test]
    fn no_loops() {
        let aba = Aba::default();
        let loops = loops_of(&aba).count();
        assert_eq!(loops, 0);

        let mut loops = loops_of(&aba);
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
    }

    #[test]
    fn a_single_loop() {
        let aba = DebugAba::default()
            .with_assumption('a', 'q')
            .with_rule('p', ['q'])
            .with_rule('q', ['p'])
            .with_rule('p', ['a']);
        let the_loop = aba.forward_set(['p', 'q'].into_iter().collect()).unwrap();
        let loops = loops_of(aba.aba()).count();
        assert_eq!(loops, 1);

        let mut loops = loops_of(&aba.aba());
        let first = loops.next().unwrap();
        assert_eq!(first.heads, the_loop);
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
    }

    #[test]
    fn two_loops() {
        let aba = DebugAba::default()
            .with_assumption('a', 'q')
            .with_rule('p', ['a'])
            .with_rule('q', ['p'])
            .with_rule('p', ['q'])
            .with_rule('r', ['q'])
            .with_rule('p', ['r']);
        let first_loop = aba.forward_set(['p', 'q'].into_iter().collect()).unwrap();
        let second_loop = aba
            .forward_set(['p', 'q', 'r'].into_iter().collect())
            .unwrap();
        let loops = loops_of(aba.aba()).count();
        assert_eq!(loops, 2);

        let mut loops = loops_of(&aba.aba());
        let next = loops.next().unwrap();
        assert!(next.heads == first_loop || next.heads == second_loop);
        let next = loops.next().unwrap();
        assert!(next.heads == first_loop || next.heads == second_loop);
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
    }

    #[test]
    fn three_loops() {
        let aba = DebugAba::default()
            .with_assumption('a', 'q')
            .with_rule('p', ['a'])
            .with_rule('q', ['p'])
            .with_rule('p', ['q'])
            .with_rule('r', ['q'])
            .with_rule('p', ['r'])
            .with_rule('r', ['p']);
        let expected = [
            aba.forward_set(['p', 'q'].into_iter().collect()).unwrap(),
            aba.forward_set(['p', 'q', 'r'].into_iter().collect())
                .unwrap(),
            aba.forward_set(['p', 'r'].into_iter().collect()).unwrap(),
        ];
        let mut loops = loops_of(&aba.aba());
        for _number in 0..expected.len() {
            let next = loops.next().unwrap();
            assert!(
                expected.contains(&next.heads),
                "Unexpected loop {:?}",
                next.heads
            );
        }
        // The iterator should be empty now
        assert!(matches!(loops.next(), None));
        assert!(matches!(loops.next(), None));
    }
}
