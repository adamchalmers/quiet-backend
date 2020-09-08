mod api;
mod config;
mod datastore;
mod metrics;
mod twoface;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate guard;
#[macro_use]
extern crate diesel;

use crate::config::Config;
use crate::datastore::postgres::PostgresStore;
use actix_service::Service;
use actix_web::{dev::ServiceResponse, middleware, web, App, HttpServer};
use datastore::postgres;
use futures::future::FutureExt;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, Level};

#[allow(clippy::cognitive_complexity)]
fn main() {
    let args: Vec<_> = std::env::args().collect();
    guard!(let [_, config_file_path, ..] = &args[..] else {
        eprintln!("First argument should be path to config file");
        return
    });

    let config= Config::from_file(config_file_path);

    // Set up logger output
    let subscriber_builder = tracing_subscriber::fmt().with_max_level(Level::DEBUG);
    if config.human_logs {
        subscriber_builder.init();
    } else {
        subscriber_builder.json().init();
    }

    info!("starting onething");

    let sys = actix_rt::System::new("onething");

    // Build the postgres client
    let db = PostgresStore::new(
        postgres::Dsn::new(&config),
        config.db_pool_size,
        Duration::from_secs(config.db_connection_timeout),
    )
    .expect("couldn't connect to Postgres");
    prometheus::register(Box::new(db.clone())).expect("couldn't register DB metrics");

    // Build the userfacing app state
    let db_pointer = Arc::new(db);
    let disable_auth = config.disable_auth;
    if disable_auth {
        warn!("Auth is disabled. This should only happen in testing.");
    }
    let state = api::State {
        ds: Arc::clone(&db_pointer),
    };

    // Start the userfacing API server
    info!(
        addr = &config.userfacing_listen_address[..],
        "starting userfacing API server"
    );
    let max_body_size = config.max_body_size;
    HttpServer::new(move || {
        App::new()
            // Middleware for Prometheus
            .wrap_fn(|request, srv| srv.call(request).map(increment_response_metrics))
            .data(state.clone())
            // enable logger
            .wrap(middleware::Logger::default())
            // limit size of the payload (global configuration)
            .data(web::JsonConfig::default().limit(max_body_size))
            .service(web::scope("/accounts").configure(api::userfacing::configure::<PostgresStore>))
            .service(web::scope("/admin").configure(api::admin::configure::<PostgresStore>))
    })
    .bind(config.userfacing_listen_address.clone())
    .expect("couldn't start userfacing HTTP server")
    .run();

    // Start the metrics server
    info!(
        addr = &config.metrics_address[..],
        "starting metrics server"
    );
    HttpServer::new(|| {
        App::new().service(
            web::scope("/metrics")
                .service(web::resource("/").route(web::get().to(metrics::endpoint::gather)))
                .service(web::resource("").route(web::get().to(metrics::endpoint::gather))),
        )
    })
    .bind(config.metrics_address)
    .expect("couldn't start metrics server")
    .run();

    sys.run().expect("actix runtime terminated");
}

/// If response is OK, increment the metrics for HTTP statuses.
fn increment_response_metrics<E, B>(
    response: Result<ServiceResponse<B>, E>,
) -> Result<ServiceResponse<B>, E> {
    match response {
        Ok(response) => {
            metrics::HTTP_RESPONSES
                .with_label_values(&[response.status().as_str()])
                .inc();
            Ok(response)
        }
        other => other,
    }
}
