mod errors;
pub mod post_store;
use crate::config::Config;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use prometheus::{
    core::{Collector, Desc},
    proto::MetricFamily,
    IntGauge, Opts,
};
use std::time::Duration;

pub struct Dsn {
    secret: String,
}

impl Dsn {
    pub fn new(config: &Config) -> Self {
        Dsn {
            secret: config.db_dsn.clone(),
        }
    }
}

impl From<Dsn> for String {
    fn from(dsn: Dsn) -> String {
        dsn.secret
    }
}

/// An implementation of datastore::PostStore backed by Postgres
#[derive(Clone)]
pub struct PostgresStore {
    pool: Pool<ConnectionManager<PgConnection>>,
    idle_conns: IntGauge,
    conns: IntGauge,
}

impl PostgresStore {
    pub fn new(
        dsn: Dsn,
        max_pool_size: u32,
        conn_timeout: Duration,
    ) -> Result<Self, anyhow::Error> {
        let manager = ConnectionManager::<PgConnection>::new(dsn);
        let pool = Pool::builder()
            .max_size(max_pool_size)
            .connection_timeout(conn_timeout)
            .build(manager)?;
        let idle_conns = IntGauge::with_opts(Opts::new(
            "quietbackend_db_connections_idle",
            "How many DB connections are currently idle",
        ))?;
        let conns = IntGauge::with_opts(Opts::new(
            "quietbackend_db_connections",
            "How many DB connections are open",
        ))?;
        Ok(Self {
            pool,
            idle_conns,
            conns,
        })
    }
}

impl Collector for PostgresStore {
    fn desc(&self) -> Vec<&Desc> {
        let mut descs = self.idle_conns.desc();
        descs.extend(self.conns.desc());
        descs
    }

    fn collect(&self) -> Vec<MetricFamily> {
        self.idle_conns
            .set(self.pool.state().idle_connections as i64);
        self.conns.set(self.pool.state().connections as i64);
        let mut metrics = self.idle_conns.collect();
        metrics.extend(self.conns.collect());
        metrics
    }
}
