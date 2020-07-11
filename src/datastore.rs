#[cfg(test)]
pub mod mock;
pub mod postfilters;
pub mod postgres;
pub mod structs;
pub mod tables;

use crate::datastore::structs::{NewPost, Post, User};
use crate::twoface::Fallible;
use async_trait::async_trait;
use postfilters::PostFilters;
use uuid::Uuid;

#[async_trait]
/// The interface for storing post data.
pub trait Client: Clone {
    async fn new_post(&self, new_post: NewPost) -> Fallible<Post>;
    async fn list_posts(&self, filters: PostFilters) -> Fallible<Vec<Post>>;
    async fn find_post(&self, user_id: Uuid, post_id: Uuid) -> Fallible<Option<Post>>;
    async fn delete_post(&self, user_id: Uuid, post_id: Uuid) -> Fallible<Option<Post>>;
    async fn timeline(&self, user_id: Uuid, num_posts: u8) -> Fallible<Vec<Post>>;
    async fn get_user(&self, user_id: Uuid) -> Fallible<Option<User>>;
}
