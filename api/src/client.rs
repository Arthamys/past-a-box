use crate::common::config::Config;
use crate::error::{Error, Result};
use serde_json;
use std::io::Read;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Client {
    ipc: Arc<Mutex<UnixStream>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Clipping,
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

    pub fn request_clipping(&mut self) {
        let mut guard = self.ipc.lock().unwrap();
        let msg = serde_json::to_string(&Request::Clipping).expect("could not serialize request");
        guard
            .write(&msg.as_bytes())
            .expect("could not write to IPC");
    }

    pub fn read_msg(&mut self) {
        let mut guard = self.ipc.lock().unwrap();
        let mut rsp = vec![0; 10];
        guard.read_exact(&mut rsp).unwrap();
        info!("response: {:?}", &rsp);
    }
}
