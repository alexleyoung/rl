use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::{Row, SqlitePool};

use crate::{
    api::dto::{SearchHitDto, SearchQueryDto, SearchResponseDto},
    error::AppError,
};

pub async fn search(
    State(pool): State<SqlitePool>,
    Query(q): Query<SearchQueryDto>,
) -> Result<Json<SearchResponseDto>, AppError> {
    let query = q.q.trim().to_string();
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let hits = if query.is_empty() { vec![] } else { run_fts(&pool, &query, limit).await? };
    Ok(Json(SearchResponseDto { query, hits }))
}

async fn run_fts(pool: &SqlitePool, q: &str, limit: i64) -> Result<Vec<SearchHitDto>, AppError> {
    let fts_q = format!("{q}*");
    let rows = sqlx::query(
        r#"SELECT source_kind, source_id, title,
               snippet(search_fts, 3, '[', ']', '…', 10) AS snip
           FROM search_fts
           WHERE search_fts MATCH ?
           ORDER BY rank
           LIMIT ?"#,
    )
    .bind(fts_q)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut hits = Vec::with_capacity(rows.len());
    for r in rows {
        let source_kind: String = r.try_get("source_kind").unwrap_or_default();
        let source_id: i64 = r.try_get("source_id").unwrap_or_default();
        let title: String = r.try_get("title").unwrap_or_default();
        let snippet: String = r.try_get("snip").unwrap_or_default();

        let (resource_id, note_id) = match source_kind.as_str() {
            "note" => {
                let row = sqlx::query!("SELECT resource_id FROM notes WHERE id=?", source_id)
                    .fetch_optional(pool).await?;
                (row.map(|r| r.resource_id).unwrap_or(0), Some(source_id))
            }
            _ => (source_id, None),
        };

        hits.push(SearchHitDto {
            source_kind,
            source_id,
            resource_id,
            note_id,
            title,
            snippet,
        });
    }
    Ok(hits)
}
