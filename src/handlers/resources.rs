use axum::{
    extract::{Form, Path, Query, State},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    indexing::{pdf as pdf_indexer, url as url_indexer},
    models::{note, resource, tag},
    views::resources as view,
};

#[derive(Deserialize)]
pub struct TagQuery { pub tag: Option<String> }

pub async fn list(
    State(pool): State<SqlitePool>,
    Query(q): Query<TagQuery>,
) -> Result<impl IntoResponse, AppError> {
    let resources = match &q.tag {
        Some(t) => resource::list_by_tag(&pool, t).await?,
        None    => resource::list(&pool).await?,
    };
    let all_tags = tag::list_all(&pool).await?;
    Ok(view::list_page(&resources, &all_tags, q.tag.as_deref()))
}

pub async fn new_form() -> impl IntoResponse {
    view::new_page()
}

#[derive(Deserialize)]
pub struct ResourceForm {
    pub kind: String,
    pub title: String,
    pub author: Option<String>,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub tags: Option<String>,
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Form(form): Form<ResourceForm>,
) -> Result<impl IntoResponse, AppError> {
    let input = resource::ResourceInput {
        kind: form.kind,
        title: form.title,
        author: non_empty(form.author),
        url: non_empty(form.url),
        file_path: non_empty(form.file_path),
        tags: form.tags,
    };
    let id = resource::create(&pool, &input).await?;
    let tags: Vec<String> = input.tags.as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    resource::set_tags(&pool, id, &tags).await?;
    // Spawn PDF indexing in background if a local file is set
    if let Some(fp) = &input.file_path {
        let fp = fp.clone();
        let pool2 = pool.clone();
        tokio::spawn(async move {
            pdf_indexer::index_pdf(&pool2, id, &fp).await;
        });
    }
    // Index article/blog URL content
    let kind = input.kind.as_str();
    if matches!(kind, "article" | "blog") {
        if let Some(u) = &input.url {
            let u = u.clone();
            let pool2 = pool.clone();
            tokio::spawn(async move {
                url_indexer::index_url(&pool2, id, &u).await;
            });
        }
    }
    Ok(Redirect::to(&format!("/resources/{id}")))
}

pub async fn show(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&pool, id).await?;
    let notes = note::list_for_resource(&pool, id).await?;
    Ok(view::detail_page(&r, &tags, &notes))
}

pub async fn edit_form(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    let tags = resource::get_tags(&pool, id).await?;
    Ok(view::edit_page(&r, &tags))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Form(form): Form<ResourceForm>,
) -> Result<impl IntoResponse, AppError> {
    let input = resource::ResourceInput {
        kind: form.kind,
        title: form.title,
        author: non_empty(form.author),
        url: non_empty(form.url),
        file_path: non_empty(form.file_path),
        tags: form.tags,
    };
    resource::update(&pool, id, &input).await?;
    let tags: Vec<String> = input.tags.as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    resource::set_tags(&pool, id, &tags).await?;
    // Re-index PDF if file path is set
    if let Some(fp) = &input.file_path {
        let fp = fp.clone();
        let pool2 = pool.clone();
        tokio::spawn(async move {
            pdf_indexer::index_pdf(&pool2, id, &fp).await;
        });
    }
    // Re-index article/blog URL
    let kind = input.kind.as_str();
    if matches!(kind, "article" | "blog") {
        if let Some(u) = &input.url {
            let u = u.clone();
            let pool2 = pool.clone();
            tokio::spawn(async move {
                url_indexer::index_url(&pool2, id, &u).await;
            });
        }
    }
    Ok(Redirect::to(&format!("/resources/{id}")))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    resource::delete(&pool, id).await?;
    Ok(Redirect::to("/"))
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.and_then(|v| if v.trim().is_empty() { None } else { Some(v) })
}
