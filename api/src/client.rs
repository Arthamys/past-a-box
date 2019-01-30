use crate::common::config::Config;
use crate::error::{Error, Result};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Client {
    ipc: Arc<Mutex<UnixStream>>,
}

impl Client {
    //! A client to the API server.
    //! It wraps functionality to communicate with the past-a-box daemon, to
    //! allow for ease of use.

    /// Create a new API client
    /// The communication with the API server is made through the `socket`
    /// configuration variable
    pub fn new() -> Result<Client> {
        let config = Config::parse()?;
        debug!("Connecting to {}", &config.socket);
        let sock = match UnixStream::connect(&config.socket) {
            Ok(s) => s,
            Err(e) => {
                error!("Could not connect to socket '{}'", &config.socket);
                return Err(Error::Io(e));
            }
        };
        debug!("Connected to {}", &config.socket);
        Ok(Client {
            ipc: Arc::new(Mutex::new(sock)),
        })
    }
}
