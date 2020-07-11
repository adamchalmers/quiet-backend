use crate::datastore::tables::users;
use crate::datastore::{postfilters::PostFilters, tables::posts};
use chrono::{offset::Utc, DateTime};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user of the website.
#[derive(Queryable, Identifiable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub name: String,
}

impl User {
    /// Has this post been deleted?
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// Parameters for the database statement which inserts new users.
#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub name: String,
}

/// A post from a user
#[derive(
    Queryable, Identifiable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Associations,
)]
#[belongs_to(User)]
pub struct Post {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub content: Content,
    pub text: String,
    pub user_id: Uuid,
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
        if let Some(user_id) = filters.user_id {
            if user_id != self.user_id {
                return false;
            }
        }
        if let Some(user_id) = &filters.user_id {
            if user_id != &self.user_id {
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
    pub user_id: Uuid,
}

#[cfg(test)]
mod post_tests {
    use super::*;
    use std::thread::sleep;
    use uuid::Uuid;

    #[test]
    fn test_post_condition() {
        let post_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let active_post = Post {
            id: post_id,
            user_id,
            text: "example text".to_owned(),
            content: Content::None,
            created_at: Utc::now(),
            deleted_at: None,
        };

        assert!(active_post.matches(&PostFilters {
            user_id: Some(user_id),
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
