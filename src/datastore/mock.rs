use crate::datastore::{postfilters::PostFilters, NewPost, Post};
use crate::twoface::Fallible;
use async_trait::async_trait;
use chrono::offset::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

type Store<T> = Arc<Mutex<Vec<T>>>;

/// A mock implementation of datastore::PostStore
#[derive(Clone, Default, Debug)]
pub struct PostStore {
    posts: Store<Post>,
}

impl PostStore {
    pub fn set_posts(&mut self, posts: Vec<Post>) {
        self.posts = Arc::new(Mutex::new(posts));
    }
}

#[async_trait]
impl super::PostStore for PostStore {
    async fn new_post(&self, new_post: NewPost) -> Fallible<Post> {
        // Insert the new post
        let t = Post {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            deleted_at: None,
            account_id: new_post.account_id,
            content: new_post.content,
            text: new_post.text,
        };
        self.posts.lock().unwrap().push(t.clone());

        Ok(t)
    }

    async fn list_posts(&self, filters: PostFilters) -> Fallible<Vec<Post>> {
        let all_posts = self.posts.lock().unwrap();
        let posts = all_posts.iter().filter(|t| t.matches(&filters));
        let mut results = Vec::new();
        for post in posts {
            results.push(post.clone())
        }
        Ok(results)
    }

    async fn find_post(&self, account_id: Uuid, uuid: Uuid) -> Fallible<Option<Post>> {
        let filters = PostFilters {
            id: Some(uuid),
            account_id: Some(account_id),
            ..Default::default()
        };
        let post = self
            .posts
            .lock()
            .unwrap()
            .iter()
            .find(|t| t.matches(&filters))
            .cloned();

        guard!(let Some(post) = post else {
            return Ok(None)
        });

        Ok(Some(post))
    }

    async fn delete_post(&self, account_id: Uuid, uuid: Uuid) -> Fallible<Option<Post>> {
        let filters = PostFilters {
            id: Some(uuid),
            account_id: Some(account_id),
            ..Default::default()
        };
        let post = self
            .posts
            .lock()
            .unwrap()
            .iter_mut()
            .find(|t| t.matches(&filters))
            .map(|post| {
                post.deleted_at = Some(Utc::now());
                post.clone()
            });
        Ok(post)
    }
}
