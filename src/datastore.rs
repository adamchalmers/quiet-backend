#[cfg(test)]
pub mod mock;
pub mod postfilters;
pub mod postgres;
pub mod tables;

use crate::datastore::tables::posts;
use crate::twoface::Fallible;
use async_trait::async_trait;
use chrono::{offset::Utc, DateTime};
use diesel_derive_enum::DbEnum;
use postfilters::PostFilters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[async_trait]
/// The interface for storing post data.
pub trait PostStore: Clone {
    async fn new_post(&self, new_post: NewPost) -> Fallible<Post>;
    async fn list_posts(&self, filters: PostFilters) -> Fallible<Vec<Post>>;
    async fn find_post(&self, account_id: Uuid, uuid: Uuid) -> Fallible<Option<Post>>;
    async fn delete_post(&self, account_id: Uuid, uuid: Uuid) -> Fallible<Option<Post>>;
}
/// A post from a user
#[derive(Queryable, Identifiable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Post {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub content: Content,
    pub text: String,
    pub account_id: Uuid,
}

#[derive(DbEnum, Debug, PartialEq, Serialize, Deserialize, Clone, Copy, Eq, Hash)]
pub enum Content {
    None,
}

impl Post {
    /// Has this post been deleted?
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    #[allow(dead_code, clippy::nonminimal_bool)]
    /// Does this post match all specified filters?
    pub fn matches(&self, filters: &PostFilters) -> bool {
        if let Some(account_id) = filters.account_id {
            if account_id != self.account_id {
                return false;
            }
        }
        if let Some(account_id) = &filters.account_id {
            if account_id != &self.account_id {
                return false;
            }
        }
        if let Some(uuid) = &filters.id {
            if uuid != &self.id {
                return false;
            }
        }
        if let Some(is_deleted) = filters.is_deleted {
            if is_deleted != self.is_deleted() {
                return false;
            }
        }
        if let Some(substring) = &filters.text_contains {
            if !self.text.contains(substring) {
                return false;
            }
        }
        if let Some(existed_at) = filters.existed_at {
            if let Some(deleted_at) = self.deleted_at {
                if !(self.created_at < existed_at && existed_at < deleted_at) {
                    return false;
                }
            } else if !(self.created_at < existed_at) {
                return false;
            }
        }
        true
    }
}

/// Parameters for the database statement which inserts new posts.
#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost {
    pub content: Content,
    pub text: String,
    pub account_id: Uuid,
}

#[cfg(test)]
mod post_tests {
    use super::*;
    use std::thread::sleep;
    use uuid::Uuid;

    #[test]
    fn test_post_condition() {
        let post_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        let active_post = Post {
            id: post_id,
            account_id,
            text: "example text".to_owned(),
            content: Content::None,
            created_at: Utc::now(),
            deleted_at: None,
        };

        assert!(active_post.matches(&PostFilters {
            account_id: Some(account_id),
            ..Default::default()
        }));

        assert!(active_post.matches(&PostFilters {
            text_contains: Some("ample".to_owned()),
            ..Default::default()
        }));

        assert!(active_post.matches(&PostFilters {
            existed_at: Some(Utc::now()),
            ..Default::default()
        }));

        assert!(active_post.matches(&PostFilters {
            is_deleted: Some(false),
            ..Default::default()
        }));

        let inactive_post = Post {
            deleted_at: Some(Utc::now()),
            ..active_post
        };
        sleep(std::time::Duration::from_micros(10));
        // The post is no longer active, so it shouldn't exist at `now()`
        assert!(!inactive_post.matches(&PostFilters {
            existed_at: Some(Utc::now()),
            ..Default::default()
        }));
    }
}
