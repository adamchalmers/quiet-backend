use serde::Deserialize;

/// Config, from CLI args or env vars.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// <address>:<port> to serve userfacing endpoints
    pub userfacing_listen_address: String,

    /// <address>:<port> to serve admin endpoints
    pub admin_listen_address: String,

    /// <address>:<port> to serve metrics on
    pub metrics_address: String,

    /// By default, output JSON logs. Only if this flag is set to true, output colourful human-friendly logs
    pub human_logs: bool,

    /// Max HTTP body size the API accepts
    #[serde(default = "max_body_size")]
    pub max_body_size: usize,

    /// password to connect to database.
    pub db_dsn: String,

    /// maximum number of connections maintained by PostgresStore
    pub db_pool_size: u32,

    /// maximum seconds waiting for a database connection
    pub db_connection_timeout: u64,

    /// Whether to disable the auth header checks in the user- and edge-facing API. This should only
    /// be true in test environments.
    pub disable_auth: bool,
}

impl Config {
    /// Will crash if file isn't found or config is invalid.
    pub fn from_file(filepath: &str) -> Self {
        let contents = std::fs::read_to_string(filepath).expect("Couldn't read from config file");
        toml::from_str(&contents).expect("couldn't parse config file")
    }
}

fn max_body_size() -> usize {
    65536
}