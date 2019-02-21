#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate api;
extern crate env_logger;
extern crate libc;
extern crate wayland_client;
extern crate wayland_protocols;

mod clipboard;
mod error;
mod wayland;

use api::client::Request;
use api::common::clipping::Clipping;
use api::server::Response;
use api::server::Server;
use std::sync::{Arc, Mutex};
use std::thread;

lazy_static! {
    static ref DAEMON: Mutex<Daemon> = Mutex::new(Daemon::new());
}

struct Daemon {
    storage: Arc<Mutex<Vec<Clipping>>>,
}

impl Daemon {
    /// Create a new Daemon instance
    pub fn new() -> Daemon {
        Daemon {
            storage: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a new clipping to the daemon's clipping storage
    fn add_clipping(&self, clip: Clipping) {
        let mut clippings = self.storage.lock().expect("Could not lock storage mutex");
        clippings.push(clip);
    }
}

fn main() {
    env_logger::init();
    info!("Starting API server!");

    let api_hdl = thread::Builder::new()
        .name("api_listener".into())
        .spawn(move || {
            let srv = Server::new(Box::new(api_handler)).unwrap();
            srv.run();
        })
        .expect("Could not spawn api server thread");

    let clipboard_hdl = clipboard::Clipboard::new_clipboard_thread();

    api_hdl.join().expect("api_server_thread crashed");
    clipboard_hdl.join().expect("clipboard_thread crashed");
}

// the handler will need to have access to the daemon storage
fn api_handler(_rq: Request) -> Response {
    let rsp = Vec::new();
    Response::Clippings(rsp)
}
