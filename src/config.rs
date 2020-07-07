use serde::Serialize;
use structopt::StructOpt;

/// Config, from CLI args or env vars.
#[derive(Debug, StructOpt, Clone, Serialize)]
#[structopt(name = "QuietBackend", about = "Backend for quiet")]
pub struct Config {
    /// <address>:<port> to serve userfacing endpoints
    #[structopt(long, env)]
    pub userfacing_listen_address: String,

    /// <address>:<port> to serve admin endpoints
    #[structopt(long, env)]
    pub admin_listen_address: String,

    /// <address>:<port> to serve metrics on
    #[structopt(long, env)]
    pub metrics_address: String,

    /// By default, output JSON logs. Only if this flag is set to true, output colourful human-friendly logs
    #[structopt(long)]
    pub human_logs: bool,

    /// Max HTTP body size the API accepts
    #[structopt(long, env, default_value = "65536")]
    pub max_body_size: usize,

    /// password to connect to database.
    /// `skip_serializing` ensures that this flag won't be printed by a json logger
    #[structopt(long, env)]
    #[serde(skip_serializing)]
    pub db_dsn: String,

    /// maximum number of connections maintained by PostgresStore
    #[structopt(long, env)]
    pub db_pool_size: u32,

    /// maximum seconds waiting for a database connection
    #[structopt(long, env)]
    pub db_connection_timeout: u64,

    /// Whether to disable the auth header checks in the user- and edge-facing API. This should only
    /// be true in test environments.
    #[structopt(long, env)]
    pub disable_auth: bool,
}
