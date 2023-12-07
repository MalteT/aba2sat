use thiserror::Error;

pub type Result<T = (), E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("while parsing: {_0}")]
    Parse(#[from] nom::Err<nom::error::Error<String>>),
    #[error("while opening the aba file: {_0}")]
    OpeningAbaFile(::std::io::Error),
    #[error("sat call interrupted")]
    SatCallInterrupted,
    #[error("problem internal check failed: {_0}")]
    ProblemCheckFailed(String),
    #[error("formatting: {_0}")]
    Format(#[from] std::fmt::Error),
    #[error("outputting: {_0}")]
    Output(#[from] std::io::Error),
}

impl From<nom::Err<nom::error::Error<&'_ str>>> for Error {
    fn from(value: nom::Err<nom::error::Error<&'_ str>>) -> Self {
        Error::from(value.map_input(|input| input.to_owned()))
    }
}
