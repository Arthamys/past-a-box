use crate::common::config::Config;
use crate::error::{Error, Result};
use std::os::unix::net::UnixListener;
use std::sync::{Arc, Mutex};

/// The Server is meant to be used by the daemon that will hold the clippings
/// to communicate to the clients.
/// It encapsulates the IPC between the client and the server.
/// To communicate with the clients, simply register a handler that
pub struct Server {
    listener: Arc<Mutex<UnixListener>>,
    on: bool,
}

impl Server {
    /// Create a new server that will listen on a specific unix domain socket
    pub fn new() -> Result<Server> {
        let cfg = Config::parse()?;
        let socket_addr = cfg.socket;
        let listener = match UnixListener::bind(&socket_addr) {
            Ok(sock) => sock,
            Err(e) => {
                error!("Could not bind to address '{}'", &socket_addr);
                return Err(Error::Io(e));
            }
        };
        Ok(Server {
            listener: Arc::new(Mutex::new(listener)),
            on: true,
        })
    }

    /// run will start listening for connections, and accept new ones
    pub fn run(&self) {
        while self.on == true {
            let guard = self.listener.lock();
            match guard.unwrap().accept() {
                Ok((_sock, addr)) => {
                    info!("New connection from {:?}", addr);
                    //handle this new connection
                }
                Err(e) => {
                    error!("Could not accept incoming connection. Error: {}", e);
                }
            }
        }
    }
}
