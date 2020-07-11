use crate::datastore;
use crate::metrics;
use crate::twoface::Fallible;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

pub mod admin;
pub mod userfacing;

/// Shared app state which can be read by any handler.
#[derive(Clone, Debug)]
pub struct State<DS: datastore::Client> {
    pub ds: Arc<DS>,
}

/// Just a named pair that can be extracted from the path of many endpoints.
#[derive(Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub struct AccountPost {
    pub user_id: Uuid,
    pub post_id: Uuid,
}

pub trait CoerceColl<T>
where
    Self: IntoIterator<Item = T>,
{
    fn coerce_into<U: From<T>>(self) -> Vec<U>;
}

impl<T> CoerceColl<T> for Vec<T> {
    fn coerce_into<U: From<T>>(self) -> Vec<U> {
        self.into_iter().map(|v| v.into()).collect()
    }
}

#[cfg(test)]
/// Parses the response into HTTP 200 containing JSON of type T. Panics otherwise.
fn parse_resp_bytes(resp: &actix_web::dev::ServiceResponse) -> &bytes::Bytes {
    let ok = resp.status() == actix_web::http::StatusCode::OK;

    if !ok {
        if let Some(actix_web::body::Body::Bytes(bytes)) = resp.response().body().as_ref() {
            if let Ok(body) = String::from_utf8(bytes.to_vec()) {
                panic!("Response status {}, body {}", resp.status(), body)
            }
        }
        panic!("HTTP Error status {}", resp.status())
    }

    match resp.response().body().as_ref() {
        Some(actix_web::body::Body::Bytes(bytes)) => bytes,
        _ => panic!("Response error"),
    }
}

#[cfg(test)]
/// Parses the response into whatever T the programmer wants, or panics.
fn parse_resp<'a, T: serde::Deserialize<'a>>(resp: &'a actix_web::dev::ServiceResponse) -> T {
    let response_body = parse_resp_bytes(resp);
    let parsed_resp: T =
        serde_json::from_slice(&response_body).expect("response was the wrong type");
    parsed_resp
}

/// Execute the closure, then log its operational metrics, e.g. time taken, whether it returned Ok/Err, etc.
async fn observe<F, Fut, R>(name: &'static str, f: F) -> Fallible<R>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Fallible<R>>,
{
    let start = Instant::now();
    let return_val = f().await;
    let duration = start.elapsed();
    metrics::HANDLER_SECS
        .with_label_values(&[name])
        .observe(duration.as_secs_f64());
    metrics::RESPONSES
        .with_label_values(&[name, variant_name(&return_val)])
        .inc();
    return_val
}

fn variant_name<T, E>(result: &Result<T, E>) -> &'static str {
    if result.is_ok() {
        "ok"
    } else {
        "err"
    }
}
