use std::sync::Arc;

use sqlx::SqlitePool;

use super::{chunk::chunk_text, embed::reindex_chunks, embed::Embedder};

pub async fn index_note(pool: &SqlitePool, embedder: Arc<Embedder>, note_id: i64, body_md: &str) {
    if body_md.trim().is_empty() {
        return;
    }
    let chunks = chunk_text(body_md);
    if chunks.is_empty() {
        return;
    }
    reindex_chunks(pool, embedder, "note", note_id, chunks).await;
}
