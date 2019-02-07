#[macro_use]
extern crate log;
extern crate api;
extern crate libc;
extern crate wayland_client;
extern crate wayland_protocols;

mod error;
mod wayland;

use api::client::Request;
use api::common::clipping::Clipping;
use api::server::Response;
use api::server::Server;
use std::sync::{Arc, Mutex};
use std::thread;
use wayland::WaylandContext;

struct Daemon {
    storage: Arc<Mutex<Vec<Clipping>>>,
}

impl Daemon {
    /// Create a new Daemon instance
    fn new() -> Daemon {
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
    println!("Starting API server!");

    let api_server_hdl = thread::Builder::new()
        .name("api_listener".into())
        .spawn(move || {
            let srv = Server::new(Box::new(handler)).unwrap();
            srv.run();
        })
        .expect("Could not spawn api server thread");

    let clipboard_hdl = thread::Builder::new()
        .name("clipboard listener".into())
        .spawn(|| {
            let ctx = WaylandContext::new();
            /*listener.listen();*/
        })
        .expect("Could not spawn clipboard listener thread");

    api_server_hdl.join().expect("api_server_thread crashed");
    clipboard_hdl.join().expect("clipboard_thread crashed");
}

// the handler will need to have access to the daemon storage
fn handler(_rq: Request) -> Response {
    let rsp = Vec::new();
    Response::Clippings(rsp)
}
