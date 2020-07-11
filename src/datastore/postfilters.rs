//! Ways to filter posts based on their fields. Filter semantics work just like SQL:
//! If a field is unset, its filter won't be applied.
//! If set, filter out posts that don't match the filter.
use chrono::{offset::Utc, DateTime};
use serde::Deserialize;
use uuid::Uuid;

/// Filters that can be applied to queries on the datastore.
#[derive(Default, Deserialize, Debug, Eq, PartialEq)]
pub struct PostFilters {
    pub is_deleted: Option<bool>,
    pub text_contains: Option<String>,
    pub existed_at: Option<DateTime<Utc>>,
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    /// Maximum number of posts to let match the filter
    #[serde(default = "default_limit")]
    pub limit: u8,
}

fn default_limit() -> u8 {
    100
}
