use std::collections::HashSet;

use crate::aba::{
    debug::DebugAba,
    problems::{
        admissibility::{EnumerateAdmissibleExtensions, VerifyAdmissibleExtension},
        complete::DecideCredulousComplete,
        conflict_free::ConflictFreeness,
    },
};

fn simple_aba_example_1() -> DebugAba {
    DebugAba::default()
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
        (set![], true),
        (set!['a'], true),
        (set!['b'], true),
        (set!['c'], true),
        (set!['a', 'b'], true),
        (set!['a', 'c'], true),
        (set!['b', 'c'], true),
        (set!['a', 'b', 'c'], false),
    ];

    set_checks
        .into_iter()
        .for_each(|(assumptions, expectation): (HashSet<char>, _)| {
            eprintln!("Checking set {assumptions:?}");
            let translated = aba.forward_set(assumptions.clone()).unwrap();
            let result = crate::aba::problems::solve(
                ConflictFreeness {
                    assumptions: translated,
                },
                aba.aba().clone(),
            )
            .unwrap();
            assert!(
                result == expectation,
                "Expected {expectation} from solver, but got {result} while checking {assumptions:?}"
            );
        })
}

#[test]
fn simple_admissible_verification() {
    let aba = simple_aba_example_1();
    let set_checks = vec![
        (set![], true),
        (set!['a', 'b'], false),
        (set!['a'], false),
        (set!['b'], true),
    ];
    set_checks
        .into_iter()
        .for_each(|(assumptions, expectation): (HashSet<char>, _)| {
            eprintln!("Checking set {assumptions:?}");
            let translated= aba.forward_set(assumptions.clone()).unwrap();
            let result =
                crate::aba::problems::solve(VerifyAdmissibleExtension { assumptions: translated }, aba.aba().clone()).unwrap();
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
    let result = crate::aba::problems::multishot_solve(
        EnumerateAdmissibleExtensions::default(),
        aba.aba().clone(),
    )
    .unwrap();
    for elem in aba.forward_sets(expected.clone()).unwrap() {
        assert!(
            result.contains(&elem),
            "{elem:?} was expected but not found in result"
        );
    }
    for elem in aba.backward_sets(result).unwrap() {
        assert!(
            expected.contains(&elem),
            "{elem:?} was found in the result, but is not expected!"
        );
    }
}

#[test]
fn simple_admissible_example_with_defense() {
    let aba = DebugAba::default()
        .with_assumption('a', 'r')
        .with_assumption('b', 's')
        .with_assumption('c', 't')
        .with_rule('p', ['q', 'a'])
        .with_rule('q', [])
        .with_rule('r', ['b', 'c'])
        .with_rule('t', vec!['a', 'b']);
    let expected: Vec<HashSet<char>> = vec![set!(), set!('a', 'b'), set!('b'), set!('b', 'c')];
    let result = crate::aba::problems::multishot_solve(
        EnumerateAdmissibleExtensions::default(),
        aba.aba().clone(),
    )
    .unwrap();
    for elem in aba.forward_sets(expected.clone()).unwrap() {
        assert!(
            result.contains(&elem),
            "{elem:?} was expected but not found in result"
        );
    }
    for elem in aba.backward_sets(result).unwrap() {
        assert!(
            expected.contains(&elem),
            "{elem:?} was found in the result, but is not expected!"
        );
    }
}

#[test]
fn simple_admissible_atomic() {
    let aba = DebugAba::default()
        .with_assumption('a', 'p')
        .with_assumption('b', 'q')
        .with_assumption('c', 'r')
        .with_rule('p', ['b'])
        .with_rule('q', ['a', 'c']);
    let expected: Vec<HashSet<char>> =
        vec![set!(), set!('b'), set!('c'), set!('a', 'c'), set!('b', 'c')];
    let result = crate::aba::problems::multishot_solve(
        EnumerateAdmissibleExtensions::default(),
        aba.aba().clone(),
    )
    .unwrap();
    for elem in aba.forward_sets(expected.clone()).unwrap() {
        assert!(
            result.contains(&elem),
            "{elem:?} was expected but not found in result"
        );
    }
    for elem in aba.backward_sets(result).unwrap() {
        assert!(
            expected.contains(&elem),
            "{elem:?} was found in the result, but is not expected!"
        );
    }
}

#[test]
fn a_chain_with_no_beginning() {
    // found this while grinding against ASPforABA (5aa9201)
    let aba = DebugAba::default()
        .with_assumption('a', 'b')
        .with_assumption('b', 'c')
        .with_rule('c', ['a', 'd'])
        .with_rule('d', ['c']);
    let expected: Vec<HashSet<char>> = vec![set!(), set!('b')];
    // 'a' cannot be defended against b since c is not derivable
    let result = crate::aba::problems::multishot_solve(
        EnumerateAdmissibleExtensions::default(),
        aba.aba().clone(),
    )
    .unwrap();
    for elem in aba.forward_sets(expected.clone()).unwrap() {
        assert!(
            result.contains(&elem),
            "{elem:?} was expected but not found in result"
        );
    }
    for elem in aba.backward_sets(result).unwrap() {
        assert!(
            expected.contains(&elem),
            "{elem:?} was found in the result, but is not expected!"
        );
    }
}

#[test]
fn loops_and_conflicts() {
    let aba = DebugAba::default()
        .with_assumption('a', 'b')
        .with_rule('b', ['a'])
        .with_rule('b', ['c'])
        .with_rule('c', ['b'])
        .with_rule('d', ['b']);
    let element = aba.forward_atom('d').unwrap();
    let result =
        crate::aba::problems::solve(DecideCredulousComplete { element }, aba.aba().clone())
            .unwrap();
    assert!(!result, "d cannot be credulous complete");
}

#[test]
fn loops_and_conflicts_2() {
    let aba = DebugAba::default()
        .with_assumption('a', 'e')
        .with_assumption('b', 'f')
        .with_rule('c', ['d'])
        .with_rule('d', ['c'])
        .with_rule('d', ['a']);
    let element = aba.forward_atom('b').unwrap();
    let result =
        crate::aba::problems::solve(DecideCredulousComplete { element }, aba.aba().clone())
            .unwrap();
    assert!(result, "b is credulous complete");
}
