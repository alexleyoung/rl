use sqlx::SqlitePool;

#[allow(dead_code)]
pub async fn list_all(pool: &SqlitePool) -> sqlx::Result<Vec<String>> {
    let rows = sqlx::query!("SELECT name FROM tags ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| r.name).collect())
}
