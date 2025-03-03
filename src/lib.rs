#![feature(iter_collect_into)]
#![feature(iter_intersperse)]
#![feature(doc_notable_trait)]

#[cfg(test)]
macro_rules! set {
    ($($elem:expr),*) => {{
        vec![$($elem),*].into_iter().collect()
    }}
}

lazy_static::lazy_static! {
    pub static ref STOP_LOOP_COUNTING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

pub mod aba;
pub mod clauses;
pub mod error;
pub mod literal;
pub mod mapper;
pub mod parser;

#[cfg(test)]
mod tests;

use std::sync::{atomic::AtomicBool, Arc};

pub use error::{Error, Result};
