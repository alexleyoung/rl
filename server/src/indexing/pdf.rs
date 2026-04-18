use std::sync::Arc;

use sqlx::SqlitePool;
use tracing::warn;

use super::{chunk::chunk_text, embed::{reindex_chunks, Embedder}, pymupdf};

pub async fn index_pdf(pool: &SqlitePool, embedder: Option<Arc<Embedder>>, resource_id: i64, file_path: &str) {
    let path = file_path.to_string();
    let content = tokio::task::spawn_blocking(move || pymupdf::extract_content(&path)).await;

    let content = match content {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => { warn!("pymupdf failed for {file_path}: {e}"); return; }
        Err(e) => { warn!("spawn_blocking failed: {e}"); return; }
    };

    // Flatten text blocks for FTS/vector indexing; images are skipped.
    let text: String = content.pages.iter().enumerate().map(|(i, page)| {
        let page_text: String = page.blocks.iter().filter_map(|b| match b {
            pymupdf::PdfBlock::Heading { text, .. } => Some(text.as_str()),
            pymupdf::PdfBlock::Paragraph { text } => Some(text.as_str()),
        }).collect::<Vec<_>>().join("\n");
        if i == 0 { page_text } else { format!("\n\n{page_text}") }
    }).collect();

    if text.trim().is_empty() {
        return;
    }

    let chunks = chunk_text(&text);

    // FTS indexing
    let _ = sqlx::query("DELETE FROM search_fts WHERE source_kind='pdf' AND source_id=?")
        .bind(resource_id)
        .execute(pool)
        .await;

    for (i, chunk) in chunks.iter().enumerate() {
        let title = format!("PDF chunk {}", i + 1);
        if let Err(e) = sqlx::query(
            "INSERT INTO search_fts(source_kind, source_id, title, body) VALUES ('pdf', ?, ?, ?)"
        )
        .bind(resource_id)
        .bind(&title)
        .bind(chunk)
        .execute(pool)
        .await
        {
            warn!("FTS insert failed for chunk {i}: {e}");
            break;
        }
    }

    tracing::info!("FTS: indexed {} PDF chunks for resource {resource_id}", chunks.len());

    // Vector indexing
    if let Some(emb) = embedder {
        reindex_chunks(pool, emb, "pdf", resource_id, chunks).await;
    }
}
