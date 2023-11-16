use crate::aba::{
    problems::{ConflictFreeness, VerifyAdmissibility},
    Aba,
};

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
        (vec![], true),
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

#[test]
fn simple_admissible_verification() {
    let aba = Aba::new()
        .with_assumption('a', 'c')
        .with_assumption('b', 'd')
        .with_rule('c', vec!['a'])
        .with_rule('c', vec!['b'])
        .with_rule('d', vec!['a']);
    let set_checks = vec![
        (vec![], true),
        (vec!['a', 'b'], false),
        (vec!['a'], false),
        (vec!['b'], true),
    ];
    set_checks
        .into_iter()
        .for_each(|(assumptions, expectation)| {
            eprintln!("Checking set {assumptions:?}");
            let result = crate::aba::problems::solve(VerifyAdmissibility { assumptions }, &aba);
            assert!(result == expectation);
        })
}
