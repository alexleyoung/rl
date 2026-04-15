use sqlx::SqlitePool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: i64,
    pub resource_id: i64,
    pub title: String,
    pub body_md: String,
    pub body_html: String,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct NoteInput {
    pub title: String,
    pub body_md: String,
    pub body_html: String,
}

pub async fn list_for_resource(pool: &SqlitePool, resource_id: i64) -> sqlx::Result<Vec<Note>> {
    sqlx::query_as!(Note,
        "SELECT id, resource_id, title, body_md, body_html, updated_at
         FROM notes WHERE resource_id=? ORDER BY updated_at DESC",
        resource_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get(pool: &SqlitePool, id: i64) -> sqlx::Result<Option<Note>> {
    sqlx::query_as!(Note,
        "SELECT id, resource_id, title, body_md, body_html, updated_at
         FROM notes WHERE id=?",
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn create(pool: &SqlitePool, resource_id: i64, input: &NoteInput) -> sqlx::Result<i64> {
    let row = sqlx::query!(
        "INSERT INTO notes (resource_id, title, body_md, body_html, updated_at)
         VALUES (?, ?, ?, ?, unixepoch()) RETURNING id",
        resource_id, input.title, input.body_md, input.body_html
    )
    .fetch_one(pool)
    .await?;
    Ok(row.id)
}

pub async fn update(pool: &SqlitePool, id: i64, input: &NoteInput) -> sqlx::Result<()> {
    sqlx::query!(
        "UPDATE notes SET title=?, body_md=?, body_html=?, updated_at=unixepoch()
         WHERE id=?",
        input.title, input.body_md, input.body_html, id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM notes WHERE id=?", id)
        .execute(pool)
        .await?;
    Ok(())
}
