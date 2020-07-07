//! For every business-logic struct in `datastore`, this module will have a matching struct
//! which redacts some business-sensitive fields.
use crate::api::{observe, AccountPost, CoerceColl, State};
use crate::datastore;
use crate::datastore::{Content, Post};
use crate::twoface::Fallible;
use actix_web::web;

use chrono::{offset::Utc, DateTime};
use serde::{self, Deserialize, Serialize};
use uuid::Uuid;

pub fn configure<DS: datastore::PostStore + 'static>(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/{account_id}/posts")
            .route("", web::post().to(write_post::<DS>))
            .route("", web::get().to(list_posts::<DS>))
            .route("/{post_id}", web::get().to(get_post::<DS>))
            .route("/{post_id}", web::delete().to(delete_post::<DS>)),
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
async fn write_post<DS: datastore::PostStore>(
    state: web::Data<State<DS>>,
    account_id: web::Path<Uuid>,
    body: web::Json<WritePostBody>,
) -> Fallible<web::Json<UserFacingPost>> {
    observe("post_post", || async {
        let new_post = datastore::NewPost {
            account_id: *account_id,
            content: body.content,
            text: body.text.clone(),
        };
        let post = state.ds.new_post(new_post).await?;
        Ok(web::Json(post.into()))
    })
    .await
}

// Get all user's posts from the datastore
async fn list_posts<DS: datastore::PostStore>(
    state: web::Data<State<DS>>,
    account_id: web::Path<Uuid>,
    filters: web::Query<PostFilters>,
) -> Fallible<web::Json<Vec<UserFacingPost>>> {
    observe("list_post", || async {
        let filters = filters.into_inner().into_datastore_filters(*account_id);
        let posts_and_conns = state.ds.list_posts(filters).await?.coerce_into();
        Ok(web::Json(posts_and_conns))
    })
    .await
}

async fn get_post<DS: datastore::PostStore>(
    state: web::Data<State<DS>>,
    path: web::Path<AccountPost>,
) -> Fallible<web::Json<Option<UserFacingPost>>> {
    observe("get_post", || async {
        let post = state.ds.find_post(path.account_id, path.post_id).await?;
        Ok(web::Json(post.map(UserFacingPost::from)))
    })
    .await
}

async fn delete_post<DS: datastore::PostStore>(
    state: web::Data<State<DS>>,
    path: web::Path<AccountPost>,
) -> Fallible<web::Json<Option<UserFacingPost>>> {
    observe("delete_post", || async {
        let response = state
            .ds
            .delete_post(path.account_id, path.post_id)
            .await?
            .map(|t| UserFacingPost::from(t));
        Ok(web::Json(response))
    })
    .await
}

#[cfg(test)]
impl UserFacingPost {
    /// Has this post been deleted?
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// Filters that users can specify via the Poststore API
#[derive(Default, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct PostFilters {
    pub name: Option<String>,
    pub is_deleted: Option<bool>,
    pub existed_at: Option<DateTime<Utc>>,
    pub uuid: Option<Uuid>,
    pub text_contains: Option<String>,
}

impl PostFilters {
    // Users should never be able to specify account ID as a filter in the API.
    // Nor should they ever be able to query posts they don't own. Instead, API filters should
    // have to be combined with an account ID (which Poststore extracts from the URL path/user creds)
    // before the datastore can execute them.
    pub fn into_datastore_filters(
        self,
        account_id: Uuid,
    ) -> crate::datastore::postfilters::PostFilters {
        crate::datastore::postfilters::PostFilters {
            account_id: Some(account_id),
            is_deleted: self.is_deleted,
            existed_at: self.existed_at,
            text_contains: self.text_contains,
            id: self.uuid,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str;
    use std::sync::Arc;

    use actix_web::{dev::Service, test, web, App, Error};
    use chrono::offset::Utc;
    use chrono::NaiveDateTime;
    use serde_json::Value as JValue;

    use userfacing::WritePostBody;

    use crate::api::{admin, parse_resp, userfacing};
    use crate::datastore::{mock, Post};

    use super::*;

    #[test]
    fn test_ser() {
        let uuid = Uuid::from_fields(42, 12, 5, &[12, 3, 9, 56, 54, 43, 8, 9]).unwrap();
        let obj = PostFilters {
            name: Some("my-post".to_owned()),
            is_deleted: Some(true),
            existed_at: Some(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(61, 0),
                Utc,
            )),
            uuid: Some(uuid),
            text_contains: Some("substring".to_owned()),
        };
        assert_eq!(
            "name=my-post&is_deleted=true&existed_at=1970-01-01T00%3A01%3A01Z&uuid=0000002a-000c-0005-0c03-0938362b0809&text_contains=substring",
            serde_qs::to_string(&obj).unwrap()
        );
    }

    #[test]
    fn test_deser() {
        // Empty query string means no filters will be applied
        assert_eq!(
            serde_qs::from_str::<PostFilters>("").unwrap(),
            PostFilters::default()
        );

        // One filter
        assert_eq!(
            serde_qs::from_str::<PostFilters>("name=example").unwrap(),
            PostFilters {
                name: Some("example".to_owned()),
                ..Default::default()
            }
        );

        // Multiple filters
        assert_eq!(
            serde_qs::from_str::<PostFilters>("is_deleted=true").unwrap(),
            PostFilters {
                is_deleted: Some(true),
                ..Default::default()
            }
        );
    }

    #[actix_rt::test]
    async fn test_new_post_can_be_viewed() -> Result<(), Error> {
        // Set up a test app
        let store = Arc::new(mock::PostStore::default());
        let mut app = test::init_service(
            App::new()
                .data(State { ds: store.clone() })
                .service(web::scope("/accounts").configure(configure::<mock::PostStore>))
                .service(web::scope("/admin").configure(admin::configure::<mock::PostStore>)),
        )
        .await;

        // Send a POST to create a new post
        let text = "something".to_owned();
        let account_id = Uuid::new_v4();
        let create_post_req = test::TestRequest::post()
            .uri(&format!("/accounts/{}/posts", account_id))
            .header("Authorization", "Bearer testbase64value")
            .set_json(&userfacing::WritePostBody {
                content: Content::None,
                text: text.clone(),
            })
            .to_request();
        let create_post_resp = app.call(create_post_req).await.unwrap();

        // Validate the response
        let created_post: UserFacingPost = parse_resp(&create_post_resp);
        assert_eq!(created_post.content, Content::None);
        assert_eq!(created_post.text, text);

        // get newly created post via user facing API
        let uri = format!("/accounts/{}/posts/{}", account_id, created_post.id);
        let get_req = test::TestRequest::get()
            .uri(&uri)
            .header("Authorization", "Bearer testbase64value")
            .to_request();
        let get_resp = app.call(get_req).await.unwrap();
        let post: UserFacingPost = parse_resp(&get_resp);
        assert_eq!(created_post, post);

        // Send a GET to list all posts
        let list_posts_req = test::TestRequest::get()
            .uri("/admin/posts")
            .header("Authorization", "Bearer testbase64value")
            .to_request();
        let list_posts_resp = app.call(list_posts_req).await.unwrap();

        // Validate the response
        let response: Vec<Post> = parse_resp(&list_posts_resp);
        assert_eq!(response[0].content, Content::None);
        assert_eq!(response[0].text, text);

        // get non-existent post
        let uri = &format!("/accounts/{}/posts/99", account_id);
        let get_req = test::TestRequest::get()
            .uri(&uri)
            .header("Authorization", "Bearer testbase64value")
            .to_request();
        let get_resp = app.call(get_req).await.unwrap();
        assert_eq!(404, get_resp.status());

        Ok(())
    }

    #[actix_rt::test]
    async fn test_get_post_filters() -> Result<(), Error> {
        // Set up a test app
        let mut app = test::init_service(
            App::new()
                .data(State {
                    ds: Arc::new(mock::PostStore::default()),
                })
                .service(web::scope("/accounts").configure(configure::<mock::PostStore>)),
        )
        .await;

        // Create several new posts in different accounts
        let account1 = Uuid::new_v4();
        let account2 = Uuid::new_v4();
        let new_posts: Vec<(WritePostBody, Uuid)> = vec![
            (
                userfacing::WritePostBody {
                    content: Content::None,
                    text: "example1".to_owned(),
                },
                account1,
            ),
            (
                userfacing::WritePostBody {
                    content: Content::None,
                    text: "example2".to_owned(),
                },
                account1,
            ),
            (
                userfacing::WritePostBody {
                    content: Content::None,
                    text: "example2".to_owned(),
                },
                account2,
            ),
        ];

        for t in new_posts {
            let req = test::TestRequest::post()
                .uri(&format!("/accounts/{}/posts", t.1))
                .header("Authorization", "Bearer testbase64value")
                .set_json(&t.0)
                .to_request();
            let response = app.call(req).await.unwrap();
            let _: UserFacingPost = parse_resp(&response);
        }

        let tests: Vec<(&'static str, usize, Uuid)> = vec![
            ("text_contains=example1", 1, account1),
            ("text_contains=example2", 1, account1),
            ("", 1, account2),
            ("text_contains=thisdoesnotexist", 0, account2),
        ];

        for test in tests {
            // Send a GET to list all posts
            let get_posts_req = test::TestRequest::get()
                .uri(&format!(
                    "{}?{}",
                    &format!("/accounts/{}/posts", test.2),
                    test.0
                ))
                .header("Authorization", "Bearer testbase64value")
                .to_request();
            let get_posts_resp = app.call(get_posts_req).await.unwrap();

            // Validate the response
            let response_posts: JValue = parse_resp(&get_posts_resp);
            if let JValue::Array(array) = response_posts {
                assert_eq!(array.len(), test.1);
            }
        }

        Ok(())
    }

    #[actix_rt::test]
    async fn test_deleting_post() -> Result<(), Error> {
        // Set up a test app with a single post already in its datastore
        let post_id = Uuid::from_fields(42, 12, 5, &[12, 3, 9, 56, 54, 43, 8, 9]).unwrap();
        let account_id = Uuid::new_v4();
        let input_post = Post {
            id: post_id,
            account_id,
            created_at: Utc::now(),
            deleted_at: None,
            content: Content::None,
            text: "example".to_owned(),
        };
        let mut ds = mock::PostStore::default();
        ds.set_posts(vec![input_post.clone()]);

        let mut app = test::init_service(
            App::new()
                .data(State { ds: Arc::new(ds) })
                .service(web::scope("/accounts").configure(configure::<mock::PostStore>)),
        )
        .await;

        // Get that single post
        let uri = format!("/accounts/{}/posts/{}", account_id, post_id);
        let get_req = test::TestRequest::get()
            .uri(&uri)
            .header("Authorization", "Bearer testbase64value")
            .to_request();
        let get_resp = app.call(get_req).await.unwrap();
        let output_post: UserFacingPost = parse_resp(&get_resp);
        assert!(!output_post.is_deleted());

        // Delete that single post
        let delete_req = test::TestRequest::delete()
            .uri(&uri)
            .header("Authorization", "Bearer testbase64value")
            .to_request();
        let delete_resp = app.call(delete_req).await.unwrap();
        let output_post: UserFacingPost = parse_resp(&delete_resp);
        assert_eq!(input_post.id, output_post.id);
        assert!(output_post.is_deleted());

        // Get all non-deleted posts. Should return an empty list.
        let uri = format!("/accounts/{}/posts?is_deleted=false", account_id);
        let get_non_deleted_req = test::TestRequest::get()
            .uri(&uri)
            .header("Authorization", "Bearer testbase64value")
            .to_request();
        let get_non_deleted_resp = app.call(get_non_deleted_req).await.unwrap();
        let output_posts: JValue = parse_resp(&get_non_deleted_resp);
        assert_eq!(output_posts, JValue::Array(Vec::new()));

        // delete non existing post, expect 404
        let different_account_id = Uuid::new_v4();
        let uri = format!("/accounts/{}/posts/99", different_account_id);
        let delete_req = test::TestRequest::delete()
            .uri(&uri)
            .header("Authorization", "Bearer testbase64value")
            .to_request();
        let delete_resp = app.call(delete_req).await.unwrap();
        assert_eq!(404, delete_resp.status());

        Ok(())
    }
}
