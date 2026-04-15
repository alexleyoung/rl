mod db;
mod error;
mod handlers;
mod indexing;
mod markdown;
mod models;
mod views;

use axum::{
    routing::{get, post},
    Router,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rl=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/rl.db".to_string());

    let pool = db::init_pool(&database_url).await.expect("failed to init db");
    tracing::info!("database ready");

    let app = Router::new()
        // Static assets
        .route("/static/*path", get(handlers::static_files::serve_static))
        // Resources
        .route("/", get(handlers::resources::list))
        .route("/resources/new", get(handlers::resources::new_form))
        .route("/resources", post(handlers::resources::create))
        .route("/resources/:id", get(handlers::resources::show))
        .route("/resources/:id/edit", get(handlers::resources::edit_form).post(handlers::resources::update))
        .route("/resources/:id/delete", post(handlers::resources::delete))
        // Notes
        .route("/resources/:rid/notes/new", get(handlers::notes::new_form))
        .route("/resources/:rid/notes", post(handlers::notes::create))
        .route("/resources/:rid/notes/:nid", get(handlers::notes::show))
        .route("/resources/:rid/notes/:nid/edit", get(handlers::notes::edit_form).post(handlers::notes::update))
        .route("/resources/:rid/notes/:nid/delete", post(handlers::notes::delete))
        // Search
        .route("/search", get(handlers::search::search_page))
        .with_state(pool);

    let bind =
        std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    tracing::info!("listening on {bind}");
    axum::serve(listener, app).await.unwrap();
}
