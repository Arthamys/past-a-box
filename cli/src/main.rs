extern crate clap;

#[macro_use]
extern crate log;

use api::client::Client;
use clap::{App, Arg, ArgGroup};

fn main() {
    env_logger::init();

    let api_client = Client::new();
    println!("api client: {:?}", api_client);
    let _test_clipping = String::from("test_clipping");
    api_client.unwrap().request_clipping();

    // Options that we want to support on the command line:
    // -l -> list
    // -h -> help
    // -t -> filter by type (not useful yet, since we only support text)
    // -c -> get the # of clippings in store
    // -p -> purge the clippings
    // -d <id> -> delete clipping <id>
    let matches = App::new("Past-a-Box cli")
        .version("0.1")
        .about("CLI to interract with the Past-a-Box daemon.")
        .author("Galilee Enguehard")
        .arg(
            Arg::with_name("list")
                .short("l")
                .long("list")
                .help("list the clippings the stored by tge daemon")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("count")
                .long("count")
                .short("c")
                .help("the number of clippings to get")
                .value_name("COUNT")
                .takes_value(true)
                .requires("list"),
        )
        .arg(
            Arg::with_name("purge")
                .short("p")
                .long("purge")
                .help("purge the clippings stored by the daemon")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("delete")
                .short("d")
                .long("delete")
                .help("delete clipping <ID> from the clippings stroed by the daemon")
                .value_name("ID")
                .takes_value(true),
        )
        .group(
            ArgGroup::with_name("commands")
                .args(&["list", "purge", "delete"])
                .required(true),
        )
        .get_matches();

    let count = matches.value_of("count").unwrap_or("15");
    let count = match count.parse::<i32>() {
        Ok(val) => val,
        Err(e) => {
            error!("Could not parse COUNT value {}", e);
            return;
        }
    };
}
