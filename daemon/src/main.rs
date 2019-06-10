#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate api;
extern crate clipboard;
extern crate env_logger;
extern crate libc;
extern crate os_pipe;
extern crate wayland_client;
extern crate wayland_protocols;

mod clippings_storage;
mod daemon;
mod error;
mod handlers;
mod wayland;

use crate::wayland::WaylandContext;
use api::client::Request;
use api::common::clipping::Clipping;
use api::server::Response;
use api::server::Server;
use daemon::Daemon;
use std::sync::{Arc, Mutex};
use std::thread;

lazy_static! {
    static ref DAEMON: Arc<Mutex<Daemon>> = Arc::new(Mutex::new(Daemon::new()));
}

fn main() {
    env_logger::init();
    info!("Starting API server!");

    let daemon = DAEMON.lock().unwrap();
    let api_storage = daemon.storage.clone();
    let wayland_storage = daemon.storage.clone();
    drop(daemon);
    let api_hdl = thread::Builder::new()
        .name("api_listener".into())
        .spawn(move || {
            let mut srv = Server::new(Box::new(move |msg| api_handler(&api_storage, msg))).unwrap();
            srv.run();
        })
        .expect("Could not spawn api server thread");

    let wayland_hdl = thread::Builder::new()
        .name("wayland thread".into())
        .spawn(move || {
            let ctx = WaylandContext::new(DAEMON.clone());
            ctx.unwrap().run();
        })
        .expect("Could not spawn wayland thread");

    wayland_hdl.join().expect("wayland_thread crashed");
    api_hdl.join().expect("api_server_thread crashed");
}

// the handler will need to have access to the daemon storage
fn api_handler(s: &Arc<Mutex<Vec<Clipping>>>, rq: Request) -> Response {
    let rsp = match rq {
        Request::Clipping => s.lock().expect("could not lock storage").to_vec(),
        Request::Purge => {
            info!("Requested to purge");
            Vec::new()
        }
        Request::Delete(id) => {
            info!("Requested to delte clipping {}", id);
            Vec::new()
        }
        _ => unimplemented!(),
    };
    info!("sending stored clipings({:?})", &rsp);
    Response::Clippings(rsp)
}
