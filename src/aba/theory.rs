use std::collections::{HashMap, HashSet};

use crate::{clauses::Clause, literal::IntoLiteral};

use super::{prepared::PreparedAba, Context};

/// Generate the logic for theory derivation in the given [`Aba`](crate::aba::Aba)
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
pub fn theory_helper<Ctx: Context>(aba: &PreparedAba) -> impl Iterator<Item = Clause> + '_ {
    // The combined list of rules, such that every
    // head is unique and possible contains a list of body rule ids
    let mut rules_combined = aba.rules.iter().enumerate().fold(
        HashMap::<_, Vec<_>>::new(),
        |mut rules, (rule_id, (head, _body))| {
            rules.entry(head).or_default().push(rule_id);
            rules
        },
    );
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
        .flat_map(|(head, rule_ids)| match &rule_ids[..] {
            // No bodies, add a clause that prevents the head from accuring in the theory
            [] => {
                vec![Clause::from(vec![Ctx::Base::from(*head).neg()])]
            }
            // A single body only, this is equivalent to a head that can only be derived by a single rule
            // H <=> RBA_rule_id
            [rule_id] => {
                vec![
                    Clause::from(vec![
                        Ctx::Base::from(*head).pos(),
                        Ctx::Rule::from(*rule_id).neg(),
                    ]),
                    Clause::from(vec![
                        Ctx::Base::from(*head).neg(),
                        Ctx::Rule::from(*rule_id).pos(),
                    ]),
                ]
            }
            // n bodies for this head
            // ```text
            //    H <=> RBA_1 or ... or RBA_n
            // ⋄  (-H or RBA_1 or ... or RBA_n) and (-RBA_1 or H) and ... (-RBA_n or H)
            // ````
            rule_ids => {
                let mut clauses = vec![];
                rule_ids
                    .iter()
                    .map(|rule_id| {
                        Clause::from(vec![
                            Ctx::Base::from(*head).pos(),
                            Ctx::Rule::from(*rule_id).neg(),
                        ])
                    })
                    .collect_into(&mut clauses);
                let last_clause = rule_ids
                    .iter()
                    .map(|rule_id| Ctx::Rule::from(*rule_id).pos())
                    .chain(std::iter::once(Ctx::Base::from(*head).neg()))
                    .collect();
                clauses.push(last_clause);
                clauses
            }
        })
}
