use crate::datastore::{
    postfilters::PostFilters,
    structs::{NewPost, Post, User},
};
use crate::twoface::Fallible;
use async_trait::async_trait;
use chrono::offset::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

type Store<T> = Arc<Mutex<Vec<T>>>;

/// A mock implementation of datastore::Client
#[derive(Clone, Default, Debug)]
pub struct Client {
    posts: Store<Post>,
}

impl Client {
    pub fn set_posts(&mut self, posts: Vec<Post>) {
        self.posts = Arc::new(Mutex::new(posts));
    }
}

#[async_trait]
impl super::Client for Client {
    async fn new_post(&self, new_post: NewPost) -> Fallible<Post> {
        // Insert the new post
        let t = Post {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            deleted_at: None,
            user_id: new_post.user_id,
            content: new_post.content,
            text: new_post.text,
        };
        self.posts.lock().unwrap().push(t.clone());

        Ok(t)
    }

    async fn list_posts(&self, filters: PostFilters) -> Fallible<Vec<Post>> {
        let all_posts = self.posts.lock().unwrap();
        let posts = all_posts
            .iter()
            .filter(|t| t.matches(&filters))
            .take(filters.limit as usize);
        let mut results = Vec::new();
        for post in posts {
            results.push(post.clone())
        }
        Ok(results)
    }

    async fn find_post(&self, user_id: Uuid, uuid: Uuid) -> Fallible<Option<Post>> {
        let filters = PostFilters {
            id: Some(uuid),
            user_id: Some(user_id),
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

    async fn delete_post(&self, user_id: Uuid, uuid: Uuid) -> Fallible<Option<Post>> {
        let filters = PostFilters {
            id: Some(uuid),
            user_id: Some(user_id),
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

    async fn timeline(&self, user_id: Uuid, num_posts: u8) -> Fallible<Vec<Post>> {
        todo!()
    }
    async fn get_user(&self, user_id: Uuid) -> Fallible<Option<User>> {
        todo!()
    }
}
