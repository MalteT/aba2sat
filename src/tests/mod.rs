use crate::aba::{problems::ConflictFreeness, Aba};

#[test]
fn simple_conflict_free_verification() {
    let aba = Aba::new()
        .with_assumption('a', 'r')
        .with_assumption('b', 's')
        .with_assumption('c', 't')
        .with_rule('p', ['q', 'a'])
        .with_rule('q', [])
        .with_rule('r', ['b', 'c']);
    let set_checks = vec![
        (vec!['a'], true),
        (vec!['b'], true),
        (vec!['c'], true),
        (vec!['a', 'b'], true),
        (vec!['a', 'c'], true),
        (vec!['b', 'c'], true),
        (vec!['a', 'b', 'c'], false),
    ];

    set_checks
        .into_iter()
        .for_each(|(assumptions, expectation)| {
            eprintln!("Checking set {assumptions:?}");
            let result = crate::aba::problems::solve(ConflictFreeness { assumptions }, &aba);
            assert!(result == expectation);
        })
}
