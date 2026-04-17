pub mod dto;
pub mod extract;
pub mod files;
pub mod notes;
pub mod resources;
pub mod search;
pub mod tags;
pub mod upload;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/resources",                  get(resources::list).post(resources::create))
        .route("/resources/:id",              get(resources::show).patch(resources::update).delete(resources::delete))
        .route("/resources/:id/quick-set",    post(resources::quick_set))
        .route("/resources/:id/tags",         post(resources::set_tags))
        .route("/resources/:id/read",         post(resources::mark_read))
        .route("/resources/:id/file",         get(files::serve))
        .route("/resources/:id/content",      get(resources::get_content))
        .route("/resources/:rid/notes",       get(notes::list).post(notes::create))
        .route("/resources/:rid/notes/:nid",  get(notes::show).patch(notes::update).delete(notes::delete))
        .route("/notes/:nid",                 get(notes::locate))
        .route("/search",                     get(search::search))
        .route("/tags",                       get(tags::list))
        .route("/upload",                     post(upload::upload).layer(DefaultBodyLimit::disable()))
        .route("/extract",                    post(extract::extract))
        .layer(cors)
}
