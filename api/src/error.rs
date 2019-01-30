use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::result;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    ConfigError(config::ConfigError),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(formatter),
            Error::ConfigError(ref err) => err.fmt(formatter),
        }
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<config::ConfigError> for Error {
    fn from(error: config::ConfigError) -> Self {
        Error::ConfigError(error)
    }
}

pub type Result<T> = result::Result<T, Error>;
