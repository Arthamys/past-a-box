use std::result;

/// Configuration fields for an API client
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// The Unix Domain Socket on which to connect to establish communication
    /// with the API server.
    /// Defaults to `/tmp/pasta.sock`
    pub socket: String,
}

impl Config {
    /// Parse the user's configuration file
    ///
    /// A configuration file is not required for the good function of the
    /// program.
    /// Configuration can be overriden through environment variables.
    /// The env variables have to start with `PAB_` and the field to override.
    pub fn parse() -> result::Result<Config, config::ConfigError> {
        let mut cfg = config::Config::default();
        cfg.set_default("socket", "/tmp/pasta.sock").unwrap();
        match cfg.merge(config::File::with_name("config.yaml")) {
            Ok(_) => (),
            Err(e) => {
                info!("gathering configuration file: {:?}", e);
            }
        };
        match cfg.merge(config::Environment::with_prefix("PAB")) {
            Ok(_) => (),
            Err(e) => {
                info!("gathering environment variables: {:?}", e);
            }
        };
        cfg.try_into()
    }
}
