use crate::common::config::Config;
use crate::error::{Error, Result};
use crate::server::Response;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Client {
    ipc: Arc<Mutex<UnixStream>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum Request {
    /// Request the list of clippings
    Clipping,

    /// Delete a single clipping
    /// #### parameter
    /// clipping_id
    Delete(usize),

    /// Purge all stored clippings
    Purge,

    /// Set active clipping
    /// #### parameter
    /// clipping_id
    Select(usize),
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

    pub fn request_clipping(&mut self) -> Result<usize> {
        self.request(Request::Clipping)
    }

    pub fn purge_clippings(&mut self) -> Result<usize> {
        self.request(Request::Purge)
    }

    pub fn delete_clipping(&mut self, id: usize) -> Result<usize> {
        self.request(Request::Delete(id))
    }

    pub fn select_clippings(&mut self, id: usize) -> Result<usize> {
        self.request(Request::Select(id))
    }

    pub fn read_msg(&mut self) -> Result<Response> {
        let guard = self.ipc.lock().unwrap();
        let decoded: bincode::Result<Response> = bincode::deserialize_from(&*guard);
        decoded.map_err(|e| Error::Bincode(e))
    }

    fn request(&mut self, req: Request) -> Result<usize> {
        let mut guard = self.ipc.lock()?;
        let encoded: Vec<u8> = bincode::serialize(&req)?;
        guard.write(&encoded).map_err(|e| Error::Io(e))
    }
}
