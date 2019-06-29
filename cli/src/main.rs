extern crate api;
extern crate clap;

#[macro_use]
extern crate log;

use api::client::Client;
use api::server::Response;
use clap::{App, Arg, ArgGroup};

fn main() {
    env_logger::init();

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
                .help("list the clippings the stored by the daemon")
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
        .arg(
            Arg::with_name("select")
                .short("s")
                .long("select")
                .help("Set clipping <ID> to be the active clipboard content")
                .value_name("ID")
                .takes_value(true),
        )
        .group(
            ArgGroup::with_name("commands")
                .args(&["list", "purge", "delete", "select"])
                .required(true),
        )
        .get_matches();

    let count = matches.value_of("count").unwrap_or("15");
    let _count = match count.parse::<i32>() {
        Ok(val) => val,
        Err(e) => {
            error!("Could not parse COUNT value {}", e);
            return;
        }
    };

    let api_client = Client::new();
    if api_client.is_err() {
        error!("could not create API client: {}", api_client.unwrap_err());
        return;
    }
    let mut client = api_client.unwrap();

    if matches.is_present("list") {
        info!("requesting clipping");
        if let Err(e) = client.request_clipping() {
            error!("could not request clippings: {}", e);
            return;
        } else {
        }
    } else if matches.is_present("purge") {
        info!("purging clippings");
        if let Err(e) = client.purge_clippings() {
            error!("could not purge clippings: {}", e);
            return;
        }
    } else if matches.is_present("delete") {
        let id = matches
            .value_of("delete")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        if let Err(e) = client.delete_clipping(id) {
            error!("could not delete clippings {}: {}", id, e);
            return;
        }
    } else if matches.is_present("select") {
        let id = matches
            .value_of("select")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        if let Err(e) = client.select_clippings(id) {
            error!("could not select clippings {}: {}", id, e);
            return;
        }
    }
    if let Ok(response) = client.read_msg() {
        if let Response::Clippings(clippings) = response {
            for clipping in clippings {
                println!("{}", clipping);
            }
        }
    } else {
        warn!("could not read response from server");
    }
}
