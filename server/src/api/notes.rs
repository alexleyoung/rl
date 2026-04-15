use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
    Json,
};

use crate::{
    api::dto::{NoteDto, NoteInputDto, NoteLocationDto},
    error::AppError,
    indexing::note as note_indexer,
    markdown,
    models::{note, resource},
    state::AppState,
};

pub async fn list(
    State(s): State<AppState>,
    Path(rid): Path<i64>,
) -> Result<Json<Vec<NoteDto>>, AppError> {
    resource::get(&s.pool, rid).await?.ok_or(AppError::NotFound)?;
    let notes = note::list_for_resource(&s.pool, rid).await?;
    Ok(Json(notes.into_iter().map(NoteDto::from).collect()))
}

pub async fn show(
    State(s): State<AppState>,
    Path((rid, nid)): Path<(i64, i64)>,
) -> Result<Json<NoteDto>, AppError> {
    let n = note::get(&s.pool, nid).await?.ok_or(AppError::NotFound)?;
    if n.resource_id != rid { return Err(AppError::NotFound); }
    Ok(Json(n.into()))
}

pub async fn create(
    State(s): State<AppState>,
    Path(rid): Path<i64>,
    Json(input): Json<NoteInputDto>,
) -> Result<impl IntoResponse, AppError> {
    resource::get(&s.pool, rid).await?.ok_or(AppError::NotFound)?;
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("title required".into()));
    }
    let body_html = markdown::render(&input.body_md);
    let ni = note::NoteInput { title: input.title, body_md: input.body_md.clone(), body_html };
    let nid = note::create(&s.pool, rid, &ni).await?;
    // Index note body asynchronously
    if let Some(emb) = s.embedder.clone() {
        let pool2 = s.pool.clone();
        let body = input.body_md.clone();
        tokio::spawn(async move { note_indexer::index_note(&pool2, emb, nid, &body).await; });
    }
    let n = note::get(&s.pool, nid).await?.ok_or(AppError::NotFound)?;
    Ok((
        StatusCode::CREATED,
        AppendHeaders([(header::LOCATION, format!("/api/v1/resources/{rid}/notes/{nid}"))]),
        Json(NoteDto::from(n)),
    ))
}

pub async fn update(
    State(s): State<AppState>,
    Path((rid, nid)): Path<(i64, i64)>,
    Json(input): Json<NoteInputDto>,
) -> Result<Json<NoteDto>, AppError> {
    let existing = note::get(&s.pool, nid).await?.ok_or(AppError::NotFound)?;
    if existing.resource_id != rid { return Err(AppError::NotFound); }
    let body_html = markdown::render(&input.body_md);
    let ni = note::NoteInput { title: input.title, body_md: input.body_md.clone(), body_html };
    note::update(&s.pool, nid, &ni).await?;
    // Re-index note body
    if let Some(emb) = s.embedder.clone() {
        let pool2 = s.pool.clone();
        let body = input.body_md.clone();
        tokio::spawn(async move { note_indexer::index_note(&pool2, emb, nid, &body).await; });
    }
    let n = note::get(&s.pool, nid).await?.ok_or(AppError::NotFound)?;
    Ok(Json(n.into()))
}

pub async fn delete(
    State(s): State<AppState>,
    Path((rid, nid)): Path<(i64, i64)>,
) -> Result<StatusCode, AppError> {
    let existing = note::get(&s.pool, nid).await?.ok_or(AppError::NotFound)?;
    if existing.resource_id != rid { return Err(AppError::NotFound); }
    note::delete(&s.pool, nid).await?;
    // Clean up embeddings too
    let _ = sqlx::query("DELETE FROM embeddings WHERE source_kind='note' AND source_id=?")
        .bind(nid)
        .execute(&s.pool)
        .await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn locate(
    State(s): State<AppState>,
    Path(nid): Path<i64>,
) -> Result<Json<NoteLocationDto>, AppError> {
    let n = note::get(&s.pool, nid).await?.ok_or(AppError::NotFound)?;
    Ok(Json(NoteLocationDto { resource_id: n.resource_id, note_id: nid }))
}
