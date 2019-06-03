#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate api;
extern crate env_logger;
extern crate libc;
extern crate os_pipe;
extern crate wayland_client;
extern crate wayland_protocols;

mod clipboard;
mod clippings_storage;
mod error;
mod handlers;
mod wayland;
mod daemon;

use daemon::Daemon;
use api::client::Request;
use api::server::Response;
use api::server::Server;
use std::sync::{Arc, Mutex};
use api::common::clipping::Clipping;
use std::thread;

lazy_static! {
    static ref DAEMON: Mutex<Daemon> = Mutex::new(Daemon::new());
}

fn main() {
    env_logger::init();
    info!("Starting API server!");

    let daemon = DAEMON.lock().unwrap();
    let api_storage = daemon.storage.clone();
    let api_hdl = thread::Builder::new()
        .name("api_listener".into())
        .spawn(move || {
            let mut srv = Server::new(Box::new(move |msg| api_handler(&api_storage, msg))).unwrap();
            srv.run();
        })
        .expect("Could not spawn api server thread");
    drop(daemon);

    let clipboard_hdl = clipboard::Clipboard::new_clipboard_thread(&DAEMON);

    clipboard_hdl.join().expect("clipboard_thread crashed");
    api_hdl.join().expect("api_server_thread crashed");
}

// the handler will need to have access to the daemon storage
fn api_handler(s: &Arc<Mutex<Vec<Clipping>>>, rq: Request) -> Response {
    let rsp = match rq {
        Request::Clipping => s.lock().expect("could not lock storage").to_vec(),
    };
    info!("sending stored clipings({:?})", &rsp);
    Response::Clippings(rsp)
}
