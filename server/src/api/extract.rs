use axum::{extract::State, Json};

use crate::{
    api::dto::{ExtractInputDto, MetadataDto},
    error::AppError,
    indexing::meta::{extract_pdf_meta, extract_url_meta},
    state::AppState,
};

pub async fn extract(
    State(_s): State<AppState>,
    Json(input): Json<ExtractInputDto>,
) -> Result<Json<MetadataDto>, AppError> {
    if let Some(path) = input.file_path {
        let meta = tokio::task::spawn_blocking(move || {
            match std::fs::read(&path) {
                Ok(bytes) => extract_pdf_meta(&bytes),
                Err(_) => MetadataDto::default(),
            }
        })
        .await
        .unwrap_or_default();
        return Ok(Json(meta));
    }

    if let Some(url) = input.url {
        let meta = extract_url_meta(&url).await;
        return Ok(Json(meta));
    }

    Err(AppError::Validation("file_path or url required".into()))
}
