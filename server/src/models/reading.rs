use sqlx::SqlitePool;

/// Upsert with status = 'ok' after a successful extraction.
pub async fn upsert_ok(
    pool: &SqlitePool,
    resource_id: i64,
    content_html: &str,
    content_text: &str,
    source_type: &str,
    word_count: i64,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO reading_content
            (resource_id, content_html, content_text, source_type,
             word_count, status, extracted_at)
         VALUES (?, ?, ?, ?, ?, 'ok', unixepoch())
         ON CONFLICT(resource_id) DO UPDATE SET
            content_html=excluded.content_html,
            content_text=excluded.content_text,
            source_type=excluded.source_type,
            word_count=excluded.word_count,
            status='ok',
            extracted_at=unixepoch()",
        resource_id, content_html, content_text, source_type, word_count
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark as pending; inserts a placeholder row if none exists.
pub async fn mark_pending(
    pool: &SqlitePool,
    resource_id: i64,
    source_type: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO reading_content (resource_id, source_type, status)
         VALUES (?, ?, 'pending')
         ON CONFLICT(resource_id) DO UPDATE SET
            source_type=excluded.source_type,
            status='pending',
            extracted_at=unixepoch()",
        resource_id, source_type
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Mark as failed so the UI can show an error with retry.
pub async fn mark_failed(
    pool: &SqlitePool,
    resource_id: i64,
    source_type: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO reading_content (resource_id, source_type, status)
         VALUES (?, ?, 'failed')
         ON CONFLICT(resource_id) DO UPDATE SET
            source_type=excluded.source_type,
            status='failed',
            extracted_at=unixepoch()",
        resource_id, source_type
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete readable content (used when URL or file_path changes).
pub async fn delete_for_resource(pool: &SqlitePool, resource_id: i64) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM reading_content WHERE resource_id=?",
        resource_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
