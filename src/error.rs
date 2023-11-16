use thiserror::Error;

pub type Result<T = (), E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("while parsing: {_0}")]
    Parse(#[from] nom::Err<nom::error::Error<String>>),
    #[error("while opening the aba file: {_0}")]
    OpeningAbaFile(::std::io::Error),
}

impl From<nom::Err<nom::error::Error<&'_ str>>> for Error {
    fn from(value: nom::Err<nom::error::Error<&'_ str>>) -> Self {
        Error::from(value.map_input(|input| input.to_owned()))
    }
}
