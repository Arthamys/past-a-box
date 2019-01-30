use api::client::Client;

fn main() {
    env_logger::init();
    let api_client = Client::new();
    println!("api client: {:?}", api_client);
}
