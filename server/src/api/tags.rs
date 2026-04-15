use axum::{extract::State, Json};

use crate::{api::dto::TagDto, error::AppError, state::AppState};

pub async fn list(State(s): State<AppState>) -> Result<Json<Vec<TagDto>>, AppError> {
    let rows = sqlx::query!(
        "SELECT t.name AS name, COUNT(rt.resource_id) AS count
         FROM tags t
         LEFT JOIN resource_tags rt ON rt.tag_id = t.id
         GROUP BY t.id
         ORDER BY t.name"
    )
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows.into_iter().map(|r| TagDto { name: r.name, count: r.count as i64 }).collect()))
}
