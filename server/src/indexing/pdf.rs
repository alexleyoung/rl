use sqlx::SqlitePool;
use tracing::warn;

const CHUNK_SIZE: usize = 1000; // characters per FTS chunk

pub async fn index_pdf(pool: &SqlitePool, resource_id: i64, file_path: &str) {
    // Run extraction on a blocking thread (pdf-extract is synchronous + CPU-bound)
    let path = file_path.to_string();
    let text = tokio::task::spawn_blocking(move || extract_text(&path)).await;

    let text = match text {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => { warn!("pdf-extract failed for {file_path}: {e}"); return; }
        Err(e)    => { warn!("spawn_blocking failed: {e}"); return; }
    };

    if text.trim().is_empty() {
        return;
    }

    // Remove stale PDF chunks for this resource before re-indexing
    let _ = sqlx::query(
        "DELETE FROM search_fts WHERE source_kind='pdf' AND source_id=?"
    )
    .bind(resource_id)
    .execute(pool)
    .await;

    // Split into chunks and insert
    let chunks = chunk_text(&text, CHUNK_SIZE);
    for (i, chunk) in chunks.iter().enumerate() {
        let title = format!("PDF chunk {}", i + 1);
        let res = sqlx::query(
            "INSERT INTO search_fts(source_kind, source_id, title, body) VALUES ('pdf', ?, ?, ?)"
        )
        .bind(resource_id)
        .bind(&title)
        .bind(chunk)
        .execute(pool)
        .await;

        if let Err(e) = res {
            warn!("FTS insert failed for chunk {i}: {e}");
            break;
        }
    }

    tracing::info!("indexed {} PDF chunks for resource {resource_id}", chunks.len());
}

fn extract_text(path: &str) -> anyhow::Result<String> {
    let bytes = std::fs::read(path)?;
    let text = pdf_extract::extract_text_from_mem(&bytes)?;
    Ok(text)
}

fn chunk_text(text: &str, size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut buf = String::with_capacity(size);
    for word in text.split_whitespace() {
        if buf.len() + word.len() + 1 > size && !buf.is_empty() {
            chunks.push(buf.trim().to_string());
            buf.clear();
        }
        if !buf.is_empty() { buf.push(' '); }
        buf.push_str(word);
    }
    if !buf.trim().is_empty() {
        chunks.push(buf.trim().to_string());
    }
    chunks
}
