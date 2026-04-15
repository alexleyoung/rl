use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};
use sqlx::SqlitePool;
use tokio_util::io::ReaderStream;

use crate::{error::AppError, models::resource};

pub async fn serve(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let r = resource::get(&pool, id).await?.ok_or(AppError::NotFound)?;
    let fp = r.file_path.ok_or(AppError::NotFound)?;

    let file = tokio::fs::File::open(&fp)
        .await
        .map_err(|e| AppError::Other(anyhow::anyhow!("cannot open {fp}: {e}")))?;
    let stream = ReaderStream::new(file);

    let mime = mime_guess::from_path(&fp).first_or_octet_stream();
    let filename = std::path::Path::new(&fp)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let disposition = format!("inline; filename=\"{filename}\"");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.as_ref())
        .header(header::CONTENT_DISPOSITION, disposition)
        .body(Body::from_stream(stream))
        .unwrap())
}
