use crate::client::Request;
use crate::common::clipping::Clipping;
use crate::common::config::Config;
use crate::error::{Error, Result};
use std::io::Read;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};

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
    on: bool,
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
            on: true,
        })
    }

    /// listen for new connections & invoke the handler on new messages
    ///
    /// This method will block the current thread, until the connection gets
    /// broken, or an error gets encountered.
    pub fn run(&self) {
        while self.on == true {
            let guard = self.listener.lock();
            for connection in guard.unwrap().incoming() {
                let mut msg = String::new();
                connection.unwrap().read_to_string(&mut msg).unwrap();
                println!("Client sent: {}", msg);
            }
            /*            match guard.unwrap().accept() {*/
            //Ok((sock, addr)) => {
            //info!("New connection from {:?}", addr);
            //let mut msg = String::new();
            //sock.read_to_string(&mut msg)
            //.expect("could not read to string");
            //}
            //Err(e) => {
            //error!("Could not accept incoming connection. Error: {}", e);
            //}
            /*}*/
        }
    }
}
