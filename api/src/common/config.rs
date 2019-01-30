use std::result;

/// Configuration fields for an API client
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// The Unix Domain Socket on which to connect to establish communication
    /// with the API server.
    pub socket: String,
}

/// Default values for the client configuration.
/// socket: '/var/run/pasta.sock'
impl Default for Config {
    fn default() -> Self {
        Config {
            socket: String::from("/var/run/pasta.sock"),
        }
    }
}

impl Config {
    /// Parse the user's configuration file
    pub fn parse() -> result::Result<Config, config::ConfigError> {
        let mut cfg = config::Config::try_from(&Config::default()).unwrap();
        match cfg.merge(config::File::with_name("config.yaml")) {
            Ok(_) => {}
            Err(_) => info!("No configuration file client."),
        };
        match cfg.merge(config::Environment::with_prefix("PAB_CLI")) {
            Ok(_) => {}
            Err(_) => info!("No environment variable overrides."),
        };
        cfg.try_into()
    }
}
