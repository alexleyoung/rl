use std::{path::PathBuf, sync::Arc};

use sqlx::SqlitePool;

use crate::indexing::embed::Embedder;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub embedder: Option<Arc<Embedder>>,
    pub upload_dir: PathBuf,
}
