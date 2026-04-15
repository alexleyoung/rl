use axum::{extract::State, Json};
use sqlx::SqlitePool;

use crate::{api::dto::TagDto, error::AppError};

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<TagDto>>, AppError> {
    let rows = sqlx::query!(
        "SELECT t.name AS name, COUNT(rt.resource_id) AS count
         FROM tags t
         LEFT JOIN resource_tags rt ON rt.tag_id = t.id
         GROUP BY t.id
         ORDER BY t.name"
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows.into_iter().map(|r| TagDto { name: r.name, count: r.count as i64 }).collect()))
}
