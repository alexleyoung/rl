use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
    Json,
};
use serde::Deserialize;

use crate::{
    api::dto::{NoteDto, QuickSetDto, ReadingContentChunkDto, ResourceDetailDto, ResourceDto, ResourceInputDto, SetTagsDto},
    error::AppError,
    indexing::{pdf as pdf_indexer, reader as reader_indexer, url as url_indexer},
    models::{note, reading, resource},
    state::AppState,
};

#[derive(Deserialize)]
pub struct TagQuery {
    pub tag: Option<String>,
    pub status: Option<String>,
}

pub async fn list(
    State(s): State<AppState>,
    Query(q): Query<TagQuery>,
) -> Result<Json<Vec<ResourceDto>>, AppError> {
    let rows = match (&q.tag, &q.status) {
        (Some(t), _) => resource::list_by_tag(&s.pool, t).await?,
        (None, Some(st)) => resource::list_by_status(&s.pool, st).await?,
        (None, None) => resource::list(&s.pool).await?,
    };
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let tags = resource::get_tags(&s.pool, r.id).await?;
        out.push(ResourceDto::from_parts(r, tags));
    }
    Ok(Json(out))
}

pub async fn show(
    State(s): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ResourceDetailDto>, AppError> {
    let r = resource::get(&s.pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&s.pool, id).await?;
    let notes = note::list_for_resource(&s.pool, id).await?;
    Ok(Json(ResourceDetailDto {
        resource: ResourceDto::from_parts(r, tags),
        notes: notes.into_iter().map(NoteDto::from).collect(),
    }))
}

pub async fn create(
    State(s): State<AppState>,
    Json(input): Json<ResourceInputDto>,
) -> Result<impl IntoResponse, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("title required".into()));
    }
    let ri = resource::ResourceInput {
        kind: input.kind.clone(),
        title: input.title,
        author: non_empty(input.author),
        url: non_empty(input.url),
        file_path: non_empty(input.file_path),
        status: non_empty(input.status),
        tags: None,
    };
    let id = resource::create(&s.pool, &ri).await?;
    resource::set_tags(&s.pool, id, &input.tags).await?;
    spawn_indexing(&s, id, ri.file_path.as_deref(), &ri.kind, ri.url.as_deref());

    let r = resource::get(&s.pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&s.pool, id).await?;
    Ok((
        StatusCode::CREATED,
        AppendHeaders([(header::LOCATION, format!("/api/v1/resources/{id}"))]),
        Json(ResourceDto::from_parts(r, tags)),
    ))
}

pub async fn update(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<ResourceInputDto>,
) -> Result<Json<ResourceDto>, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("title required".into()));
    }
    let existing = resource::get(&s.pool, id).await?.ok_or(AppError::NotFound)?;
    let ri = resource::ResourceInput {
        kind: input.kind.clone(),
        title: input.title,
        author: non_empty(input.author),
        url: non_empty(input.url),
        file_path: non_empty(input.file_path),
        status: non_empty(input.status).or(Some(existing.status)),
        tags: None,
    };
    resource::update(&s.pool, id, &ri).await?;
    resource::set_tags(&s.pool, id, &input.tags).await?;
    reading::delete_for_resource(&s.pool, id).await.ok();
    spawn_indexing(&s, id, ri.file_path.as_deref(), &ri.kind, ri.url.as_deref());

    let r = resource::get(&s.pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&s.pool, id).await?;
    Ok(Json(ResourceDto::from_parts(r, tags)))
}

pub async fn delete(
    State(s): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    resource::delete(&s.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn quick_set(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<QuickSetDto>,
) -> Result<Json<ResourceDto>, AppError> {
    let r = resource::get(&s.pool, id).await?.ok_or(AppError::NotFound)?;
    let val = input.value.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    match input.field.as_str() {
        "url" => {
            sqlx::query!("UPDATE resources SET url=? WHERE id=?", val, id)
                .execute(&s.pool).await?;
            reading::delete_for_resource(&s.pool, id).await.ok();
            if matches!(r.kind.as_str(), "article" | "blog") {
                if let Some(u) = val.clone() {
                    let s2 = s.clone();
                    let u2 = u.clone();
                    tokio::spawn(async move { url_indexer::index_url(&s2.pool, s2.embedder, id, &u).await; });
                    let pool = s.pool.clone();
                    tokio::spawn(async move { reader_indexer::extract_url(&pool, id, &u2).await; });
                }
            }
        }
        "file_path" => {
            sqlx::query!("UPDATE resources SET file_path=? WHERE id=?", val, id)
                .execute(&s.pool).await?;
            reading::delete_for_resource(&s.pool, id).await.ok();
            if let Some(fp) = val.clone() {
                let s2 = s.clone();
                let fp2 = fp.clone();
                tokio::spawn(async move { pdf_indexer::index_pdf(&s2.pool, s2.embedder, id, &fp).await; });
                let pool = s.pool.clone();
                tokio::spawn(async move { reader_indexer::extract_pdf(&pool, id, &fp2).await; });
            }
        }
        "status" => {
            let status = val.ok_or_else(|| AppError::Validation("status required".into()))?;
            if !matches!(status.as_str(), "inbox" | "reading" | "queue" | "done") {
                return Err(AppError::Validation(format!("invalid status: {status}")));
            }
            resource::set_status(&s.pool, id, &status).await?;
        }
        other => return Err(AppError::Validation(format!("unknown field: {other}"))),
    }
    let r = resource::get(&s.pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&s.pool, id).await?;
    Ok(Json(ResourceDto::from_parts(r, tags)))
}

pub async fn mark_read(
    State(s): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    resource::get(&s.pool, id).await?.ok_or(AppError::NotFound)?;
    resource::touch_last_read(&s.pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ContentRange {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_content(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Query(r): Query<ContentRange>,
) -> Result<Json<Option<ReadingContentChunkDto>>, AppError> {
    resource::get(&s.pool, id).await?.ok_or(AppError::NotFound)?;
    let Some(c) = reading::get(&s.pool, id).await? else {
        return Ok(Json(None));
    };

    let blocks = reader_indexer::split_top_level_blocks(&c.content_html);
    let total = blocks.len() as i64;
    let offset = r.offset.unwrap_or(0).max(0).min(total);
    let max_blocks = r.limit.unwrap_or(200).clamp(1, 2000);
    const TARGET_BYTES: usize = 50_000;

    let mut taken = 0i64;
    let mut bytes = 0usize;
    while offset + taken < total && taken < max_blocks {
        let b = blocks[(offset + taken) as usize];
        if taken > 0 && bytes + b.len() > TARGET_BYTES {
            break;
        }
        bytes += b.len();
        taken += 1;
    }
    let start = offset as usize;
    let end = (offset + taken) as usize;
    let html = blocks[start..end].join("\n");

    Ok(Json(Some(ReadingContentChunkDto {
        status: c.status,
        source_type: c.source_type,
        word_count: c.word_count,
        total_blocks: total,
        offset,
        next_offset: offset + taken,
        has_more: offset + taken < total,
        html,
    })))
}

pub async fn set_tags(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Json(input): Json<SetTagsDto>,
) -> Result<Json<ResourceDto>, AppError> {
    resource::get(&s.pool, id).await?.ok_or(AppError::NotFound)?;
    let tags: Vec<String> = input.tags.iter()
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    resource::set_tags(&s.pool, id, &tags).await?;
    let r = resource::get(&s.pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&s.pool, id).await?;
    Ok(Json(ResourceDto::from_parts(r, tags)))
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.and_then(|v| if v.trim().is_empty() { None } else { Some(v) })
}

fn spawn_indexing(s: &AppState, id: i64, file_path: Option<&str>, kind: &str, url: Option<&str>) {
    if let Some(fp) = file_path {
        let fp = fp.to_string();
        let s2 = s.clone();
        let fp2 = fp.clone();
        tokio::spawn(async move { pdf_indexer::index_pdf(&s2.pool, s2.embedder, id, &fp).await; });
        let pool = s.pool.clone();
        tokio::spawn(async move { reader_indexer::extract_pdf(&pool, id, &fp2).await; });
    }
    if matches!(kind, "article" | "blog") {
        if let Some(u) = url {
            let u = u.to_string();
            let s2 = s.clone();
            let u2 = u.clone();
            tokio::spawn(async move { url_indexer::index_url(&s2.pool, s2.embedder, id, &u).await; });
            let pool = s.pool.clone();
            tokio::spawn(async move { reader_indexer::extract_url(&pool, id, &u2).await; });
        }
    }
}
