use std::io;
use std::result;
use wayland_client::{ConnectError, GlobalError};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    WaylandConnectError(ConnectError),
    WaylandGlobalError(GlobalError),
    IoError(io::Error),
}

impl From<ConnectError> for Error {
    fn from(e: ConnectError) -> Self {
        Error::WaylandConnectError(e)
    }
}

impl From<GlobalError> for Error {
    fn from(e: GlobalError) -> Self {
        Error::WaylandGlobalError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::IoError(e)
    }
}
