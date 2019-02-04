extern crate api;

use api::client::Request;
use api::server::Response;
use api::server::Server;

fn main() {
    println!("Starting API server!");
    let srv = Server::new(Box::new(zob)).unwrap();
    srv.run();
}

fn zob(rq: Request) -> Response {
    let rsp = Vec::new();
    Response::Clippings(rsp)
}
