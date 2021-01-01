use crate::api::Database;
use crate::datastore::{postfilters::PostFilters, structs::Post};
use crate::twoface::Fallible;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/posts").route(web::get().to(list_all_posts)));
}

// Admin endpoint
async fn list_all_posts(
    state: web::Data<Database>,
    filters: web::Query<PostFilters>,
) -> Fallible<web::Json<Vec<Post>>> {
    let data = state.ds.list_posts(filters.0).await?;
    Ok(web::Json(data))
}
