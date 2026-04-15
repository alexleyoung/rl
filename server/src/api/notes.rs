use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
    Json,
};
use sqlx::SqlitePool;

use crate::{
    api::dto::{NoteDto, NoteInputDto, NoteLocationDto},
    error::AppError,
    markdown,
    models::{note, resource},
};

pub async fn list(
    State(pool): State<SqlitePool>,
    Path(rid): Path<i64>,
) -> Result<Json<Vec<NoteDto>>, AppError> {
    resource::get(&pool, rid).await?.ok_or(AppError::NotFound)?;
    let notes = note::list_for_resource(&pool, rid).await?;
    Ok(Json(notes.into_iter().map(NoteDto::from).collect()))
}

pub async fn show(
    State(pool): State<SqlitePool>,
    Path((rid, nid)): Path<(i64, i64)>,
) -> Result<Json<NoteDto>, AppError> {
    let n = note::get(&pool, nid).await?.ok_or(AppError::NotFound)?;
    if n.resource_id != rid {
        return Err(AppError::NotFound);
    }
    Ok(Json(n.into()))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Path(rid): Path<i64>,
    Json(input): Json<NoteInputDto>,
) -> Result<impl IntoResponse, AppError> {
    resource::get(&pool, rid).await?.ok_or(AppError::NotFound)?;
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("title required".into()));
    }
    let body_html = markdown::render(&input.body_md);
    let ni = note::NoteInput { title: input.title, body_md: input.body_md, body_html };
    let nid = note::create(&pool, rid, &ni).await?;
    let n = note::get(&pool, nid).await?.ok_or(AppError::NotFound)?;
    Ok((
        StatusCode::CREATED,
        AppendHeaders([(header::LOCATION, format!("/api/v1/resources/{rid}/notes/{nid}"))]),
        Json(NoteDto::from(n)),
    ))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path((rid, nid)): Path<(i64, i64)>,
    Json(input): Json<NoteInputDto>,
) -> Result<Json<NoteDto>, AppError> {
    let existing = note::get(&pool, nid).await?.ok_or(AppError::NotFound)?;
    if existing.resource_id != rid {
        return Err(AppError::NotFound);
    }
    let body_html = markdown::render(&input.body_md);
    let ni = note::NoteInput { title: input.title, body_md: input.body_md, body_html };
    note::update(&pool, nid, &ni).await?;
    let n = note::get(&pool, nid).await?.ok_or(AppError::NotFound)?;
    Ok(Json(n.into()))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path((rid, nid)): Path<(i64, i64)>,
) -> Result<StatusCode, AppError> {
    let existing = note::get(&pool, nid).await?.ok_or(AppError::NotFound)?;
    if existing.resource_id != rid {
        return Err(AppError::NotFound);
    }
    note::delete(&pool, nid).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn locate(
    State(pool): State<SqlitePool>,
    Path(nid): Path<i64>,
) -> Result<Json<NoteLocationDto>, AppError> {
    let n = note::get(&pool, nid).await?.ok_or(AppError::NotFound)?;
    Ok(Json(NoteLocationDto { resource_id: n.resource_id, note_id: nid }))
}
