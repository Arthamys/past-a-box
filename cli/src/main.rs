use api::client::Client;

fn main() {
    env_logger::init();
    let mut api_client = Client::new();
    println!("api client: {:?}", api_client);
    let test_clipping = String::from("test_clipping");
    api_client.unwrap().request_clipping();
}
