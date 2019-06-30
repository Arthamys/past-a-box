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

mod daemon;
mod error;
mod handlers;
mod wayland;

use crate::wayland::WaylandContext;
use api::client::Request;
use api::common::clipping::Clipping;
use api::server::Response;
use api::server::Server;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
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
            let ctx = WaylandContext::new();
            ctx.unwrap().run();
        })
        .expect("Could not spawn wayland thread");

    wayland_hdl.join().expect("wayland_thread crashed");
    api_hdl.join().expect("api_server_thread crashed");
}

// the handler will need to have access to the daemon storage
fn api_handler(s: &Arc<Mutex<Vec<Clipping>>>, rq: Request) -> Response {
    let rsp = match rq {
        Request::Clipping => {
            let clippings = s
                .lock()
                .expect("could not lock storage")
                .to_vec()
                .into_iter();
            let res = clippings
                .enumerate()
                .map(|(i, mut val)| {
                    val.id = i;
                    val
                })
                .collect();
            Response::Clippings(res)
        }
        Request::Select(id) => {
            info!("Setting active clipping to {}", id);
            set_active_clippings(&s, id);
            Response::Ok
        }
        Request::Purge => {
            info!("Requested to purge");
            Response::Ok
        }
        Request::Delete(id) => {
            info!("Requested to delte clipping {}", id);
            Response::Ok
        }
    };
    info!("Response: {:?}", &rsp);
    rsp
}

fn set_active_clippings(storage: &Arc<Mutex<Vec<Clipping>>>, id: usize) {
    debug!("id: {}", id);
    let mut ctx: ClipboardContext =
        ClipboardProvider::new().expect("could not get clipboard context");
    let clip_data = storage.lock().expect("could not lock storage").to_vec()[id]
        .data
        .clone();
    if let Err(e) = ctx.set_contents(clip_data) {
        error!("could not set clipboard active content: {:?}", e);
    }
}
