use axum::{
    extract::{Multipart, State},
    Json,
};
use serde::Serialize;
use ts_rs::TS;

use crate::{error::AppError, state::AppState};

#[derive(Serialize, TS)]
#[ts(export, export_to = "client/src/lib/types/")]
pub struct UploadResponseDto {
    pub path: String,
    pub filename: String,
}

pub async fn upload(
    State(s): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponseDto>, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::Other(e.into()))? {
        let filename = field
            .file_name()
            .map(|n| sanitize_filename(n))
            .ok_or_else(|| AppError::Validation("missing filename".into()))?;

        let data = field.bytes().await.map_err(|e| AppError::Other(e.into()))?;

        // Avoid collisions by prepending a timestamp-based prefix
        let prefix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let stored_name = format!("{prefix}_{filename}");
        let dest = s.upload_dir.join(&stored_name);

        tokio::fs::write(&dest, &data)
            .await
            .map_err(|e| AppError::Other(anyhow::anyhow!("write failed: {e}")))?;

        let path = dest
            .canonicalize()
            .map_err(|e| AppError::Other(anyhow::anyhow!("canonicalize failed: {e}")))?
            .to_string_lossy()
            .to_string();

        return Ok(Json(UploadResponseDto { path, filename }));
    }

    Err(AppError::Validation("no file in request".into()))
}

fn sanitize_filename(name: &str) -> String {
    // Keep only the base name (strip any path components), replace unsafe chars
    let base = std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("upload");
    base.chars()
        .map(|c| if c.is_alphanumeric() || matches!(c, '.' | '-' | '_') { c } else { '_' })
        .collect()
}
