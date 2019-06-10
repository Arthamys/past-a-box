use std::error;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::result;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    ConfigError(config::ConfigError),
    PoisonError(String),
    Bincode(Box<bincode::ErrorKind>),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(formatter),
            Error::ConfigError(ref err) => err.fmt(formatter),
            Error::Bincode(ref err) => err.fmt(formatter),
            Error::PoisonError(ref err) => err.fmt(formatter),
        }
    }
}

impl From<Box<bincode::ErrorKind>> for Error {
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        Error::Bincode(error)
    }
}

impl From<std::sync::PoisonError<std::sync::MutexGuard<'_, std::os::unix::net::UnixStream>>>
    for Error
{
    fn from(
        error: std::sync::PoisonError<std::sync::MutexGuard<'_, std::os::unix::net::UnixStream>>,
    ) -> Self {
        Error::PoisonError("Poisoned Lock".into())
    }
}

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
