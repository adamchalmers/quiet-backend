use crate::datastore::{
    postfilters::PostFilters,
    postgres::{
        errors::{BlockingResp, DbPoolResult},
        PostgresStore,
    },
    tables::posts,
    NewPost, Post, PostStore,
};
use crate::twoface::{Fallible, TfError};
use actix_web::web::block;
use async_trait::async_trait;
use diesel::{
    dsl::now,
    expression::BoxableExpression,
    expression_methods::BoolExpressionMethods,
    pg::Pg,
    query_dsl::{QueryDsl, RunQueryDsl},
    sql_types::Bool,
    Connection, ExpressionMethods, OptionalExtension, TextExpressionMethods,
};
use uuid::Uuid;

#[async_trait]
impl PostStore for PostgresStore {
    async fn new_post(&self, new_post: NewPost) -> Fallible<Post> {
        let conn = self.pool.get()?;
        let post = block(move || {
            conn.transaction::<_, TfError, _>(|| {
                // Insert the new post
                let post: Post = diesel::insert_into(posts::table)
                    .values(&new_post)
                    .get_result(&conn)?;

                Ok(post)
            })
        })
        .await
        .to_resp()?;
        Ok(post)
    }

    async fn list_posts(&self, filters: PostFilters) -> Fallible<Vec<Post>> {
        let conn = self.pool.get()?;
        let query_result: DbPoolResult<_> = block(move || {
            // Get posts
            let mut query = posts::table.into_boxed();
            for filter in filters.as_sql_where() {
                query = query.filter(filter);
            }
            let posts = query.get_results(&conn)?;

            Ok(posts)
        })
        .await;
        Ok(query_result.to_resp()?)
    }

    async fn find_post(&self, account_id: Uuid, id: Uuid) -> Fallible<Option<Post>> {
        let conn = self.pool.get()?;
        let query_result: DbPoolResult<_> = block(move || {
            let target_post: Option<Post> = posts::table
                .find(id)
                .filter(posts::account_id.eq(account_id))
                .first(&conn)
                .optional()?;

            guard!(let Some(target_post) = target_post else {
                return Ok(None);
            });

            Ok(Some(target_post))
        })
        .await;
        Ok(query_result.to_resp()?)
    }

    async fn delete_post(&self, account_id: Uuid, id: Uuid) -> Fallible<Option<Post>> {
        let conn = self.pool.get()?;
        let post = block(move || {
            conn.transaction::<_, anyhow::Error, _>(|| {
                // Delete the post
                let target = posts::table.find(id);
                let query_result: Option<Post> = diesel::update(target)
                    .filter(posts::account_id.eq(account_id))
                    .set(posts::deleted_at.eq(now))
                    .get_result::<Post>(&conn)
                    .optional()?;

                Ok(query_result)
            })
        })
        .await
        .to_resp()?;
        Ok(post)
    }
}

impl PostFilters {
    pub fn as_sql_where(
        &self,
    ) -> Vec<Box<dyn BoxableExpression<posts::table, Pg, SqlType = Bool>>> {
        let mut wheres: Vec<Box<dyn BoxableExpression<posts::table, Pg, SqlType = Bool>>> =
            Vec::new();
        if let Some(id) = self.id {
            wheres.push(Box::new(posts::id.eq(id)))
        }
        if let Some(substring) = &self.text_contains {
            wheres.push(Box::new(posts::text.like(format!("%{}%", substring))))
        }
        if let Some(is_deleted) = self.is_deleted {
            if is_deleted {
                wheres.push(Box::new(posts::deleted_at.is_not_null()))
            } else {
                wheres.push(Box::new(posts::deleted_at.is_null()))
            }
        }
        if let Some(existed_at) = self.existed_at {
            wheres.push(Box::new(posts::created_at.lt(existed_at)));
            wheres.push(Box::new(
                posts::deleted_at
                    .is_null()
                    .or(posts::deleted_at.gt(existed_at)),
            ));
        }
        if let Some(account_id) = self.account_id {
            wheres.push(Box::new(posts::account_id.eq(account_id)))
        }
        wheres
    }
}
