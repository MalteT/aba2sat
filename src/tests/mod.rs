use std::collections::HashSet;

use crate::aba::{
    problems::{
        admissibility::{Admissibility, VerifyAdmissibility},
        conflict_free::ConflictFreeness,
    },
    Aba,
};

fn simple_aba_example_1() -> Aba<char> {
    Aba::new()
        .with_assumption('a', 'r')
        .with_assumption('b', 's')
        .with_assumption('c', 't')
        .with_rule('p', ['q', 'a'])
        .with_rule('q', [])
        .with_rule('r', ['b', 'c'])
}

#[test]
fn simple_conflict_free_verification() {
    let aba = simple_aba_example_1();
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
            let result =
                crate::aba::problems::solve(ConflictFreeness { assumptions }, &aba).unwrap();
            assert!(result == expectation);
        })
}

#[test]
fn simple_admissible_verification() {
    let aba = simple_aba_example_1();
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
            let result =
                crate::aba::problems::solve(VerifyAdmissibility { assumptions: assumptions.clone() }, &aba).unwrap();
            assert!(
                result == expectation,
                "Expected {expectation} from solver, but got {result} while checking {assumptions:?}"
            );
        })
}

#[test]
fn simple_admissible_example() {
    let aba = simple_aba_example_1();
    let expected: Vec<HashSet<char>> = vec![set!(), set!('b'), set!('b', 'c'), set!('c')];
    let result = crate::aba::problems::multishot_solve(Admissibility::default(), &aba).unwrap();
    for elem in &expected {
        assert!(
            result.contains(elem),
            "{elem:?} was expected but not found in result"
        );
    }
    for elem in &result {
        assert!(
            expected.contains(elem),
            "{elem:?} was found in the result, but is not expected!"
        );
    }
}

#[test]
fn simple_admissible_example_with_defense() {
    let aba = simple_aba_example_1().with_rule('t', vec!['a', 'b']);
    let expected: Vec<HashSet<char>> = vec![set!(), set!('a', 'b'), set!('b'), set!('b', 'c')];
    let result = crate::aba::problems::multishot_solve(Admissibility::default(), &aba).unwrap();
    for elem in &expected {
        assert!(
            result.contains(elem),
            "{elem:?} was expected but not found in result"
        );
    }
    for elem in &result {
        assert!(
            expected.contains(elem),
            "{elem:?} was found in the result, but is not expected!"
        );
    }
}
