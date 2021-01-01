//! For every business-logic struct in `datastore`, this module will have a matching struct
//! which redacts some business-sensitive fields.
use crate::api::{observe, AccountPost, CoerceColl, Database};
use crate::datastore::structs::{Content, NewPost, Post};
use crate::twoface::Fallible;
use actix_web::web;

use chrono::{offset::Utc, DateTime};
use serde::{self, Deserialize, Serialize};
use uuid::Uuid;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/{user_id}/posts")
            .route("", web::post().to(write_post))
            .route("", web::get().to(list_posts))
            .route("/{post_id}", web::get().to(get_post))
            .route("/{post_id}", web::delete().to(delete_post)),
    );
}

/// A subset of Post that doesn't include business-sensitive fields
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
pub struct UserFacingPost {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub text: String,
    pub content: Content,
}

impl From<Post> for UserFacingPost {
    // Discard business-sensitive fields to convert Post into UserFacingPost
    fn from(t: Post) -> Self {
        Self {
            id: t.id,
            created_at: t.created_at,
            deleted_at: t.deleted_at,
            content: t.content,
            text: t.text,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct WritePostBody {
    pub text: String,
    pub content: Content,
}

// Insert a post into the datastore
async fn write_post(
    state: web::Data<Database>,
    user_id: web::Path<Uuid>,
    body: web::Json<WritePostBody>,
) -> Fallible<web::Json<UserFacingPost>> {
    observe("post_post", || async {
        let new_post = NewPost {
            user_id: *user_id,
            content: body.content,
            text: body.text.clone(),
        };
        let post = state.ds.new_post(new_post).await?;
        Ok(web::Json(post.into()))
    })
    .await
}

// Get all user's posts from the datastore
async fn list_posts(
    state: web::Data<Database>,
    user_id: web::Path<Uuid>,
    filters: web::Query<PostFilters>,
) -> Fallible<web::Json<Vec<UserFacingPost>>> {
    observe("list_post", || async {
        let filters = filters.into_inner().into_datastore_filters(*user_id);
        let posts_and_conns = state.ds.list_posts(filters).await?.coerce_into();
        Ok(web::Json(posts_and_conns))
    })
    .await
}

async fn get_post(
    state: web::Data<Database>,
    path: web::Path<AccountPost>,
) -> Fallible<web::Json<Option<UserFacingPost>>> {
    observe("get_post", || async {
        let post = state.ds.find_post(path.user_id, path.post_id).await?;
        Ok(web::Json(post.map(UserFacingPost::from)))
    })
    .await
}

async fn delete_post(
    state: web::Data<Database>,
    path: web::Path<AccountPost>,
) -> Fallible<web::Json<Option<UserFacingPost>>> {
    observe("delete_post", || async {
        let response = state
            .ds
            .delete_post(path.user_id, path.post_id)
            .await?
            .map(|t| UserFacingPost::from(t));
        Ok(web::Json(response))
    })
    .await
}

/// Filters that users can specify via the Poststore API
#[derive(Default, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct PostFilters {
    pub name: Option<String>,
    pub is_deleted: Option<bool>,
    pub existed_at: Option<DateTime<Utc>>,
    pub uuid: Option<Uuid>,
    pub text_contains: Option<String>,
    pub limit: u8,
}

impl PostFilters {
    // Users should never be able to specify account ID as a filter in the API.
    // Nor should they ever be able to query posts they don't own. Instead, API filters should
    // have to be combined with an account ID (which Poststore extracts from the URL path/user creds)
    // before the datastore can execute them.
    pub fn into_datastore_filters(
        self,
        user_id: Uuid,
    ) -> crate::datastore::postfilters::PostFilters {
        crate::datastore::postfilters::PostFilters {
            user_id: Some(user_id),
            is_deleted: self.is_deleted,
            existed_at: self.existed_at,
            text_contains: self.text_contains,
            id: self.uuid,
            limit: self.limit,
        }
    }
}
