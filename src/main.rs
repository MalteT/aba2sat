#![feature(iter_collect_into)]
#![feature(iter_intersperse)]

macro_rules! lit {
    (+ $lit:ident $($($name:ident)?:$var:ident)*) => {
        {
            let constructed: $lit = $lit { $($($name:)? $var),* };
            crate::literal::IntoLiteral::pos(constructed)
        }
    };
    (- $lit:ident $($($name:ident)?:$var:ident)*) => {
        {
            let constructed: $lit = $lit { $($($name:)? $var),* };
            crate::literal::IntoLiteral::neg(constructed)
        }
    };
}

use aba::{problems::ConflictFreeness, Aba};

pub mod aba;
pub mod clauses;
pub mod literal;
pub mod mapper;
#[cfg(test)]
mod tests;

fn main() {
    let aba = Aba::new()
        .with_assumption('a', 'r')
        .with_assumption('b', 's')
        .with_assumption('c', 't')
        .with_rule('p', ['q', 'a'])
        .with_rule('q', [])
        .with_rule('r', ['b', 'c'])
        .with_rule('r', ['d']);
    let result = aba::problems::solve(
        ConflictFreeness {
            assumptions: vec!['a', 'b', 'c'],
        },
        &aba,
    );
    println!("ConflictFreeness: {result}")
}
