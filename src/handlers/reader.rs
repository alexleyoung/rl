use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
};
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    indexing::reader as reader_indexer,
    models::{reading, resource, resource::Resource},
    views::reader as view,
};

/// Does this resource have a source that the reader can work with?
fn has_readable_source(r: &Resource) -> bool {
    let url_readable = r.url.is_some()
        && matches!(r.kind.as_str(), "article" | "blog" | "paper");
    url_readable || r.file_path.is_some()
}

/// Spawn a background extraction for the resource, picking URL over
/// file_path when both exist.
fn spawn_extraction(pool: &SqlitePool, r: &Resource) {
    let id = r.id;
    if let Some(u) = &r.url {
        if matches!(r.kind.as_str(), "article" | "blog" | "paper") {
            let u = u.clone();
            let pool2 = pool.clone();
            tokio::spawn(async move {
                reader_indexer::extract_url(&pool2, id, &u).await;
            });
            return;
        }
    }
    if let Some(fp) = &r.file_path {
        let fp = fp.clone();
        let pool2 = pool.clone();
        tokio::spawn(async move {
            reader_indexer::extract_pdf(&pool2, id, &fp).await;
        });
    }
}

/// GET /resources/:id/read
pub async fn read_view(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    if !has_readable_source(&r) {
        return Err(AppError::NotFound);
    }

    let content = reading::get_for_resource(&pool, id).await?;

    let markup = match content {
        Some(c) if c.status == "ok" => view::reader_page(&r, &c),
        Some(c) if c.status == "pending" => view::reader_pending_page(&r),
        Some(c) if c.status == "failed" => view::reader_failed_page(&r),
        None => {
            // No extraction attempted yet — start one and show pending.
            spawn_extraction(&pool, &r);
            view::reader_pending_page(&r)
        }
        _ => view::reader_failed_page(&r),
    };
    Ok(markup)
}

/// POST /resources/:id/read/refresh — force re-extraction.
pub async fn refresh(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    if !has_readable_source(&r) {
        return Err(AppError::NotFound);
    }
    reading::delete_for_resource(&pool, id).await?;
    spawn_extraction(&pool, &r);
    Ok(Redirect::to(&format!("/resources/{id}/read")))
}
