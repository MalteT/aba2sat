#![feature(iter_collect_into)]
#![feature(iter_intersperse)]
#![feature(doc_notable_trait)]

#[cfg(test)]
macro_rules! set {
    ($($elem:expr),*) => {{
        vec![$($elem),*].into_iter().collect()
    }}
}

pub mod aba;
pub mod clauses;
pub mod error;
pub mod literal;
pub mod mapper;
pub mod parser;

#[cfg(test)]
mod tests;

pub use error::{Error, Result};
