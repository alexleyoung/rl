mod api;
mod db;
mod error;
mod indexing;
mod markdown;
mod models;
mod state;

use std::{path::PathBuf, sync::Arc};

use axum::Router;
use state::AppState;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::indexing::embed::Embedder;

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

    // Initialise embedder (optional — if it fails we log a warning and continue
    // with FTS-only search rather than crashing the server).
    let model_cache = std::env::var("MODEL_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data/models"));
    std::fs::create_dir_all(&model_cache).ok();

    let embedder: Option<Arc<Embedder>> = match Embedder::new(model_cache) {
        Ok(e) => {
            tracing::info!("embedder ready");
            Some(Arc::new(e))
        }
        Err(e) => {
            tracing::warn!("embedder init failed (FTS-only mode): {e}");
            None
        }
    };

    let upload_dir = std::env::var("UPLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data/files"));
    std::fs::create_dir_all(&upload_dir).ok();

    let state = AppState { pool, embedder, upload_dir };

    let app = Router::new()
        .nest("/api/v1", api::router())
        .with_state(state);

    let bind = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();
    tracing::info!("listening on {bind}");
    axum::serve(listener, app).await.unwrap();
}
