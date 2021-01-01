use crate::datastore::postgres::PostgresStore;
use crate::metrics;
use crate::twoface::Fallible;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

pub mod admin;
pub mod userfacing;

#[derive(Clone)]
pub struct Database {
    pub ds: Arc<PostgresStore>,
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
