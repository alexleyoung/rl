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

    // Handle subcommands before starting the server.
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("reindex-pdfs") {
        run_reindex_pdfs().await;
        return;
    }

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/rl.db".to_string());

    let pool = db::init_pool(&database_url).await.expect("failed to init db");
    tracing::info!("database ready");

    // Probe for PyMuPDF availability; non-fatal.
    if let Err(e) = indexing::pymupdf::self_test() {
        tracing::warn!(
            "PDF extraction unavailable: {e}. PDFs will upload but not be indexed. \
             Install with: pip3 install -r server/scripts/requirements.txt"
        );
    } else {
        tracing::info!("pdf extractor ready");
    }

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

async fn run_reindex_pdfs() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/rl.db".to_string());
    let pool = db::init_pool(&database_url).await.expect("failed to init db");

    if let Err(e) = indexing::pymupdf::self_test() {
        eprintln!("error: PDF extractor unavailable: {e}");
        eprintln!("Install with: pip3 install -r server/scripts/requirements.txt");
        std::process::exit(1);
    }

    let rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT r.id, r.file_path FROM resources r \
         JOIN reading_content rc ON rc.resource_id = r.id \
         WHERE rc.source_type = 'pdf' AND r.file_path IS NOT NULL"
    )
    .fetch_all(&pool)
    .await
    .expect("db query failed");

    let total = rows.len();
    println!("Reindexing {total} PDF resource(s)…");

    for (i, (resource_id, file_path)) in rows.into_iter().enumerate() {
        println!("[{}/{}] resource {resource_id}: {file_path}", i + 1, total);

        // Reset state.
        let _ = sqlx::query("DELETE FROM search_fts WHERE source_kind='pdf' AND source_id=?")
            .bind(resource_id).execute(&pool).await;
        let _ = sqlx::query("DELETE FROM embeddings WHERE source_kind='pdf' AND source_id=?")
            .bind(resource_id).execute(&pool).await;

        indexing::reader::extract_pdf(&pool, resource_id, &file_path).await;
        indexing::pdf::index_pdf(&pool, None, resource_id, &file_path).await;
    }

    println!("Done.");
}
