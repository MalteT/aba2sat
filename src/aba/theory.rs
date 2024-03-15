use std::collections::{HashMap, HashSet};

use crate::{
    clauses::{Clause, ClauseList},
    literal::{IntoLiteral, Literal, TheoryAtom},
};

use super::{prepared::PreparedAba, Num, TheoryHelper};

/// Generate the logic for theory derivation in the given [`Aba`]
///
/// This will need a valid [`TheoryAtom`] that will be used to construct the logic
///
/// # Explanation
///
/// We will mainly operate on heads of rules here. So consider head `p` and all bodies `b`
/// in the set of all bodies of `p`: `bodies(p)`.
/// Every body `b` in `bodies(p)` is a set of atoms. Any set of atoms (any body) can be
/// used to derive `p`. So the following relation must hold:
/// - if `p` is true, at least one body `b` must be true aswell.
///   this only holds, because `p` itself is not assumption (since we're
///   only talking flat ABA)
/// - if `b` in `bodies(p)` is true, `p` must be true aswell
///
/// The entire logic in this function is required to implement this equality in DNF.
///
/// # Extra steps
///
/// - We do some extra work here to prevent atoms that never occur in the head of rule and
/// are not an assumption from ever being true.
/// - heads with a single body are common enough in practice to benefit from special care.
///   A lot of the overhead is due to the fact that multiple bodies are an option, if that's
///   not given for a head `p` we use the simplified translation logic where `p` is true iff
///   `bodies(p)` is true.
pub fn theory_helper<I: TheoryAtom>(aba: &PreparedAba) -> impl Iterator<Item = Clause> + '_ {
    // The combined list of rules, such that every
    // head is unique and possible contains a list of bodies
    let mut rules_combined =
        aba.rules
            .iter()
            .fold(HashMap::<_, Vec<_>>::new(), |mut rules, (head, body)| {
                rules.entry(head).or_default().push(body);
                rules
            });
    // All atoms that can be derived by rules
    let rule_heads: HashSet<_> = aba.rule_heads().collect();
    // For every non-assumption, that is not derivable add a rule without a body,
    // such that it cannot be derived at all. This is to prevent the solver from
    // guessing this atom on it's own
    aba.universe()
        .filter(|atom| !aba.contains_assumption(atom))
        .filter(|atom| !rule_heads.contains(atom))
        .map(|atom| (atom, vec![]))
        .collect_into(&mut rules_combined);
    // All combined rules
    // These are heads with any number of bodies, possibly none
    rules_combined
        .into_iter()
        .flat_map(|(head, bodies)| match &bodies[..] {
            // No bodies, add a clause that prevents the head from accuring in the theory
            [] => {
                vec![Clause::from(vec![I::new(*head).neg()])]
            }
            // A single body only, this is equivalent to a head that can only be derived by a single rule
            [body] => body_to_clauses::<I>(I::new(*head).pos(), body),
            // n bodies, we'll need to take extra care to allow any number of bodies to derive this
            // head without logic errors
            bodies => {
                let mut clauses = vec![];
                bodies
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, body)| {
                        body_to_clauses::<I>(TheoryHelper::<I>::new(idx, *head).pos(), body)
                    })
                    .collect_into(&mut clauses);
                let helpers: Vec<_> = (0..bodies.len())
                    .map(|idx| TheoryHelper::<I>::new(idx, *head).pos())
                    .collect();
                let mut right_implification: Clause = helpers.iter().cloned().collect();
                right_implification.push(I::new(*head).neg());
                clauses.push(right_implification);
                helpers
                    .into_iter()
                    .map(|helper| Clause::from(vec![I::new(*head).pos(), helper.negative()]))
                    .collect_into(&mut clauses);
                clauses
            }
        })
}

fn body_to_clauses<I: TheoryAtom>(head: Literal, body: &HashSet<Num>) -> ClauseList {
    let mut clauses = vec![];
    let mut left_implication: Clause = body.iter().map(|elem| I::new(*elem).neg()).collect();
    left_implication.push(head.clone().positive());
    clauses.push(left_implication);
    body.iter()
        .map(|elem| vec![head.clone().negative(), I::new(*elem).pos()].into())
        .collect_into(&mut clauses);
    clauses
}
