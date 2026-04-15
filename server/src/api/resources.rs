use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{AppendHeaders, IntoResponse},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    api::dto::{NoteDto, QuickSetDto, ResourceDetailDto, ResourceDto, ResourceInputDto},
    error::AppError,
    indexing::{pdf as pdf_indexer, url as url_indexer},
    models::{note, resource},
};

#[derive(Deserialize)]
pub struct TagQuery {
    pub tag: Option<String>,
}

pub async fn list(
    State(pool): State<SqlitePool>,
    Query(q): Query<TagQuery>,
) -> Result<Json<Vec<ResourceDto>>, AppError> {
    let rows = match &q.tag {
        Some(t) => resource::list_by_tag(&pool, t).await?,
        None => resource::list(&pool).await?,
    };
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let tags = resource::get_tags(&pool, r.id).await?;
        out.push(ResourceDto::from_parts(r, tags));
    }
    Ok(Json(out))
}

pub async fn show(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ResourceDetailDto>, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&pool, id).await?;
    let notes = note::list_for_resource(&pool, id).await?;
    Ok(Json(ResourceDetailDto {
        resource: ResourceDto::from_parts(r, tags),
        notes: notes.into_iter().map(NoteDto::from).collect(),
    }))
}

pub async fn create(
    State(pool): State<SqlitePool>,
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
        tags: None,
    };
    let id = resource::create(&pool, &ri).await?;
    resource::set_tags(&pool, id, &input.tags).await?;
    spawn_indexing(&pool, id, ri.file_path.as_deref(), &ri.kind, ri.url.as_deref());

    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&pool, id).await?;
    let dto = ResourceDto::from_parts(r, tags);
    Ok((
        StatusCode::CREATED,
        AppendHeaders([(header::LOCATION, format!("/api/v1/resources/{id}"))]),
        Json(dto),
    ))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(input): Json<ResourceInputDto>,
) -> Result<Json<ResourceDto>, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::Validation("title required".into()));
    }
    resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    let ri = resource::ResourceInput {
        kind: input.kind.clone(),
        title: input.title,
        author: non_empty(input.author),
        url: non_empty(input.url),
        file_path: non_empty(input.file_path),
        tags: None,
    };
    resource::update(&pool, id, &ri).await?;
    resource::set_tags(&pool, id, &input.tags).await?;
    spawn_indexing(&pool, id, ri.file_path.as_deref(), &ri.kind, ri.url.as_deref());

    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&pool, id).await?;
    Ok(Json(ResourceDto::from_parts(r, tags)))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    resource::delete(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn quick_set(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(input): Json<QuickSetDto>,
) -> Result<Json<ResourceDto>, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    let val = input.value.and_then(|v| {
        let t = v.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    });
    match input.field.as_str() {
        "url" => {
            sqlx::query!("UPDATE resources SET url=? WHERE id=?", val, id)
                .execute(&pool).await?;
            if matches!(r.kind.as_str(), "article" | "blog") {
                if let Some(u) = val.clone() {
                    let pool2 = pool.clone();
                    tokio::spawn(async move { url_indexer::index_url(&pool2, id, &u).await; });
                }
            }
        }
        "file_path" => {
            sqlx::query!("UPDATE resources SET file_path=? WHERE id=?", val, id)
                .execute(&pool).await?;
            if let Some(fp) = val.clone() {
                let pool2 = pool.clone();
                tokio::spawn(async move { pdf_indexer::index_pdf(&pool2, id, &fp).await; });
            }
        }
        other => return Err(AppError::Validation(format!("unknown field: {other}"))),
    }
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&pool, id).await?;
    Ok(Json(ResourceDto::from_parts(r, tags)))
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.and_then(|v| if v.trim().is_empty() { None } else { Some(v) })
}

fn spawn_indexing(pool: &SqlitePool, id: i64, file_path: Option<&str>, kind: &str, url: Option<&str>) {
    if let Some(fp) = file_path {
        let fp = fp.to_string();
        let pool2 = pool.clone();
        tokio::spawn(async move { pdf_indexer::index_pdf(&pool2, id, &fp).await; });
    }
    if matches!(kind, "article" | "blog") {
        if let Some(u) = url {
            let u = u.to_string();
            let pool2 = pool.clone();
            tokio::spawn(async move { url_indexer::index_url(&pool2, id, &u).await; });
        }
    }
}
