use crate::api::State;
use crate::datastore::{postfilters::PostFilters, Post, PostStore};
use crate::twoface::Fallible;
use actix_web::web;


pub fn configure<DS: PostStore + 'static>(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/posts").route(web::get().to(list_all_posts::<DS>)));
}

// Admin endpoint
async fn list_all_posts<DS: PostStore>(
    state: web::Data<State<DS>>,
    filters: web::Query<PostFilters>,
) -> Fallible<web::Json<Vec<Post>>> {
    let data = state.ds.list_posts(filters.0).await?;
    Ok(web::Json(data))
}
