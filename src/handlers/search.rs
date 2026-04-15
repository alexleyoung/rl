use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::{Row, SqlitePool};

use crate::{error::AppError, views::search as view};

#[derive(Deserialize)]
pub struct SearchQuery { pub q: Option<String> }

pub struct SearchHit {
    pub title: String,
    pub source_kind: String,
    pub url: String,
    pub snippet: String,
}

pub async fn search_page(
    State(pool): State<SqlitePool>,
    Query(q): Query<SearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let query = q.q.as_deref().unwrap_or("").trim().to_string();
    let hits = if query.is_empty() {
        vec![]
    } else {
        run_fts(&pool, &query).await?
    };
    Ok(view::search_page(&query, &hits))
}

async fn run_fts(pool: &SqlitePool, q: &str) -> sqlx::Result<Vec<SearchHit>> {
    let fts_q = format!("{q}*");
    // Use raw query() (not macro) to handle FTS5 virtual table columns
    let rows = sqlx::query(
        r#"SELECT source_kind, source_id, title,
               snippet(search_fts, 3, '[', ']', '…', 10) AS snip
           FROM search_fts
           WHERE search_fts MATCH ?
           ORDER BY rank
           LIMIT 50"#,
    )
    .bind(fts_q)
    .fetch_all(pool)
    .await?;

    let hits = rows.iter().map(|r| {
        let source_kind: String = r.try_get("source_kind").unwrap_or_default();
        let source_id: i64 = r.try_get("source_id").unwrap_or_default();
        let title: String = r.try_get("title").unwrap_or_default();
        let snippet: String = r.try_get("snip").unwrap_or_default();

        let url = match source_kind.as_str() {
            "resource" => format!("/resources/{source_id}"),
            "note"     => format!("/notes-redirect/{source_id}"),
            _          => format!("/resources/{source_id}"),
        };
        SearchHit { title, source_kind, url, snippet }
    }).collect();

    Ok(hits)
}
