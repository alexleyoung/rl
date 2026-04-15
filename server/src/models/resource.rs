use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Resource {
    pub id: i64,
    pub kind: String,
    pub title: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub added_at: i64,
    pub last_read_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceInput {
    pub kind: String,
    pub title: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub tags: Option<String>, // comma-separated
}

pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Resource>> {
    sqlx::query_as!(Resource,
        "SELECT id, kind, title, author, url, file_path, added_at, last_read_at
         FROM resources ORDER BY added_at DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Resource>> {
    sqlx::query_as!(Resource,
        "SELECT id, kind, title, author, url, file_path, added_at, last_read_at
         FROM resources WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, input: &ResourceInput) -> sqlx::Result<i64> {
    let row = sqlx::query!(
        "INSERT INTO resources (kind, title, author, url, file_path)
         VALUES (?, ?, ?, ?, ?) RETURNING id",
        input.kind, input.title, input.author, input.url, input.file_path
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

pub async fn update(pool: &SqlitePool, id: i64, input: &ResourceInput) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE resources SET kind=?, title=?, author=?, url=?, file_path=?
         WHERE id=?",
        input.kind, input.title, input.author, input.url, input.file_path, id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM resources WHERE id=?", id)
        .execute(pool)
        .await?;
    Ok(())
}

// Tag helpers on resources
pub async fn get_tags(pool: &SqlitePool, resource_id: i64) -> sqlx::Result<Vec<String>> {
    let rows = sqlx::query!(
        "SELECT t.name FROM tags t
         JOIN resource_tags rt ON rt.tag_id = t.id
         WHERE rt.resource_id = ?
         ORDER BY t.name",
        resource_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.name).collect())
}

pub async fn set_tags(pool: &SqlitePool, resource_id: i64, tags: &[String]) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM resource_tags WHERE resource_id=?", resource_id)
        .execute(pool)
        .await?;
    for name in tags {
        let name = name.trim();
        if name.is_empty() { continue; }
        sqlx::query!("INSERT OR IGNORE INTO tags (name) VALUES (?)", name)
            .execute(pool)
            .await?;
        sqlx::query!(
            "INSERT OR IGNORE INTO resource_tags (resource_id, tag_id)
             SELECT ?, id FROM tags WHERE name=?",
            resource_id, name
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn touch_last_read(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    sqlx::query!("UPDATE resources SET last_read_at = unixepoch() WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}

// For tag-filter on list
pub async fn list_by_tag(pool: &SqlitePool, tag: &str) -> sqlx::Result<Vec<Resource>> {
    sqlx::query_as!(Resource,
        "SELECT r.id, r.kind, r.title, r.author, r.url, r.file_path, r.added_at, r.last_read_at
         FROM resources r
         JOIN resource_tags rt ON rt.resource_id = r.id
         JOIN tags t ON t.id = rt.tag_id
         WHERE t.name = ? COLLATE NOCASE
         ORDER BY r.added_at DESC",
        tag
    )
    .fetch_all(pool)
    .await
}
