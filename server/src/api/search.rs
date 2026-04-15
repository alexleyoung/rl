use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    Json,
};
use sqlx::Row;

use crate::{
    api::dto::{SearchHitDto, SearchQueryDto, SearchResponseDto},
    error::AppError,
    indexing::embed::{blob_to_vec, cosine},
    state::AppState,
};

pub async fn search(
    State(s): State<AppState>,
    Query(q): Query<SearchQueryDto>,
) -> Result<Json<SearchResponseDto>, AppError> {
    let query = q.q.trim().to_string();
    let limit = q.limit.unwrap_or(50).clamp(1, 200) as usize;

    if query.is_empty() {
        return Ok(Json(SearchResponseDto { query, hits: vec![] }));
    }

    // Run FTS and vector search concurrently
    let (fts_res, vec_res) = tokio::join!(
        run_fts(&s.pool, &query, limit * 2),
        run_vector(&s, &query, limit * 2),
    );

    let fts_hits = fts_res?;
    let vec_hits = vec_res.unwrap_or_default(); // degrade gracefully if embedder absent

    let hits = rrf_fuse(fts_hits, vec_hits, 60, limit, &s.pool).await?;
    Ok(Json(SearchResponseDto { query, hits }))
}

// ─── FTS ────────────────────────────────────────────────────────────────────

struct FtsHit {
    source_kind: String,
    source_id: i64,
    title: String,
    snippet: String,
}

async fn run_fts(pool: &sqlx::SqlitePool, q: &str, limit: usize) -> Result<Vec<FtsHit>, AppError> {
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
    .bind(limit as i64)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(|r| FtsHit {
        source_kind: r.try_get("source_kind").unwrap_or_default(),
        source_id:   r.try_get("source_id").unwrap_or_default(),
        title:       r.try_get("title").unwrap_or_default(),
        snippet:     r.try_get("snip").unwrap_or_default(),
    }).collect())
}

// ─── Vector ─────────────────────────────────────────────────────────────────

struct VecHit {
    source_kind: String,
    source_id: i64,
    text: String,
    score: f32,
}

async fn run_vector(s: &AppState, q: &str, limit: usize) -> anyhow::Result<Vec<VecHit>> {
    let embedder = match &s.embedder {
        Some(e) => e.clone(),
        None => return Ok(vec![]),
    };

    let q_vecs = embedder.embed(vec![q.to_string()]).await?;
    let q_vec = &q_vecs[0];

    let rows = sqlx::query("SELECT source_kind, source_id, text, vector FROM embeddings")
        .fetch_all(&s.pool)
        .await?;

    let mut scored: Vec<VecHit> = rows.iter().filter_map(|r| {
        let blob: Vec<u8> = r.try_get("vector").ok()?;
        let vec = blob_to_vec(&blob);
        let score = cosine(q_vec, &vec);
        Some(VecHit {
            source_kind: r.try_get("source_kind").unwrap_or_default(),
            source_id:   r.try_get("source_id").unwrap_or_default(),
            text:        r.try_get("text").unwrap_or_default(),
            score,
        })
    }).collect();

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    Ok(scored)
}

// ─── Reciprocal rank fusion ──────────────────────────────────────────────────

async fn rrf_fuse(
    fts: Vec<FtsHit>,
    vec: Vec<VecHit>,
    k: usize,
    limit: usize,
    pool: &sqlx::SqlitePool,
) -> Result<Vec<SearchHitDto>, AppError> {
    // Key: (source_kind, source_id)
    let mut scores: HashMap<(String, i64), f32> = HashMap::new();
    let mut titles: HashMap<(String, i64), String> = HashMap::new();
    let mut snippets: HashMap<(String, i64), String> = HashMap::new();

    for (rank, h) in fts.iter().enumerate() {
        let key = (h.source_kind.clone(), h.source_id);
        *scores.entry(key.clone()).or_default() += 1.0 / (k + rank + 1) as f32;
        titles.entry(key.clone()).or_insert_with(|| h.title.clone());
        snippets.entry(key.clone()).or_insert_with(|| h.snippet.clone());
    }
    for (rank, h) in vec.iter().enumerate() {
        let key = (h.source_kind.clone(), h.source_id);
        *scores.entry(key.clone()).or_default() += 1.0 / (k + rank + 1) as f32;
        titles.entry(key.clone()).or_insert_with(|| {
            // For vector-only hits, use first 80 chars of chunk as title
            let t = h.text.chars().take(80).collect::<String>();
            if t.len() < h.text.len() { format!("{t}…") } else { t }
        });
        snippets.entry(key.clone()).or_insert_with(|| {
            let s = h.text.chars().take(200).collect::<String>();
            if s.len() < h.text.len() { format!("{s}…") } else { s }
        });
    }

    let mut ranked: Vec<((String, i64), f32)> = scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit);

    let mut hits = Vec::with_capacity(ranked.len());
    for ((source_kind, source_id), _score) in ranked {
        let (resource_id, note_id) = resolve_ids(&source_kind, source_id, pool).await;
        let title = titles.get(&(source_kind.clone(), source_id)).cloned().unwrap_or_default();
        let snippet = snippets.get(&(source_kind.clone(), source_id)).cloned().unwrap_or_default();
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

async fn resolve_ids(source_kind: &str, source_id: i64, pool: &sqlx::SqlitePool) -> (i64, Option<i64>) {
    match source_kind {
        "note" => {
            let row = sqlx::query!("SELECT resource_id FROM notes WHERE id=?", source_id)
                .fetch_optional(pool).await.ok().flatten();
            (row.map(|r| r.resource_id).unwrap_or(0), Some(source_id))
        }
        _ => (source_id, None),
    }
}
