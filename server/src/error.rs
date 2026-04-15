use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("not found")]
    NotFound,
    #[error("validation: {0}")]
    Validation(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Serialize)]
pub struct ApiError<'a> {
    pub code: &'a str,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            AppError::NotFound      => (StatusCode::NOT_FOUND,            "not_found"),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST,          "validation"),
            AppError::Db(_)         => (StatusCode::INTERNAL_SERVER_ERROR, "db"),
            AppError::Other(_)      => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
        };
        (status, Json(ApiError { code, message: self.to_string() })).into_response()
    }
}
