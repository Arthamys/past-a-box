use crate::client::Request;
use crate::common::clipping::Clipping;
use crate::common::config::Config;
use crate::error::{Error, Result};
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::{
    atomic::{self, Ordering},
    Arc, Mutex,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Clippings(Vec<Clipping>),
}

/// The Server is meant to be used by the daemon that will hold the clippings
/// to communicate to the clients.
/// It encapsulates the IPC between the client and the server.
/// To communicate with the clients, simply register a handler that will
/// dispatch the incoming client message to the function that will return the
/// response to the request.
pub struct Server {
    listener: Arc<Mutex<UnixListener>>,
    handler: Box<FnMut(Request) -> Response>,
    on: atomic::AtomicBool,
}

impl Server {
    /// Create a new server that will listen on a specific unix domain socket
    ///
    /// The address can be configured from the general past-a-box configuration
    /// file, or through the environment variable PAB_SOCKET
    pub fn new(handler: Box<FnMut(Request) -> Response>) -> Result<Server> {
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
            handler: handler,
            on: atomic::AtomicBool::new(true),
        })
    }

    /// listen for new connections & invoke the handler on new messages
    ///
    /// This method will block the current thread, until the connection gets
    /// broken, or an error gets encountered.
    pub fn run(&mut self) {
        while self.on.load(Ordering::Relaxed) {
            let guard = self.listener.lock();
            info!("Looping over incoming connections");
            for connection in guard.unwrap().incoming() {
                info!("new connection: {:?}", &connection);
                let mut msg = vec![0; 10];
                let mut co = connection.expect("could not access connection");
                //TODO: Read the messages from clients
                co.read_exact(&mut msg).expect("could not read message");
                println!("Client sent: {:?}", &msg);
                let rsp = (self.handler)(serde_json::from_slice(&msg).expect("Invalid JSON"));
                info!("response: {:?}", &rsp);
                //TODO: send out the response
                let rsp2 = serde_json::to_vec(&rsp).unwrap();
                co.write(&rsp2).expect("could not send response");
            }
        }
    }
}
