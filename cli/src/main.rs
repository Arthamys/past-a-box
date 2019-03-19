use api::client::Client;

#[macro_use]
extern crate log;

fn main() {
    env_logger::init();
    let api_client = Client::new();
    println!("api client: {:?}", api_client);
    let mut client = api_client.unwrap();
    info!("requesting clipping");
    client.request_clipping();
    info!("reading msg");
    client.read_msg();
}
