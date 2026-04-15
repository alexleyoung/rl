use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    error::AppError,
    markdown,
    models::{note, resource},
    views::notes as view,
};

pub async fn new_form(
    State(pool): State<SqlitePool>,
    Path(rid): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, rid).await?.ok_or(AppError::NotFound)?;
    Ok(view::new_page(&r))
}

#[derive(Deserialize)]
pub struct NoteForm {
    pub title: String,
    pub body_md: String,
    pub body_html: Option<String>, // may be empty; server renders
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Path(rid): Path<i64>,
    Form(form): Form<NoteForm>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, rid).await?.ok_or(AppError::NotFound)?;
    let _ = r;
    let body_html = markdown::render(&form.body_md);
    let input = note::NoteInput {
        title: form.title,
        body_md: form.body_md,
        body_html,
    };
    let nid = note::create(&pool, rid, &input).await?;
    Ok(Redirect::to(&format!("/resources/{rid}/notes/{nid}")))
}

pub async fn show(
    State(pool): State<SqlitePool>,
    Path((rid, nid)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, rid).await?.ok_or(AppError::NotFound)?;
    let n = note::get(&pool, nid).await?.ok_or(AppError::NotFound)?;
    Ok(view::view_page(&r, &n))
}

pub async fn edit_form(
    State(pool): State<SqlitePool>,
    Path((rid, nid)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, AppError> {
    let r = resource::get(&pool, rid).await?.ok_or(AppError::NotFound)?;
    let n = note::get(&pool, nid).await?.ok_or(AppError::NotFound)?;
    Ok(view::edit_page(&r, &n))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path((rid, nid)): Path<(i64, i64)>,
    Form(form): Form<NoteForm>,
) -> Result<impl IntoResponse, AppError> {
    let body_html = if form.body_html.as_deref().unwrap_or("").is_empty() {
        markdown::render(&form.body_md)
    } else {
        form.body_html.unwrap()
    };
    let input = note::NoteInput {
        title: form.title,
        body_md: form.body_md,
        body_html,
    };
    note::update(&pool, nid, &input).await?;
    Ok(Redirect::to(&format!("/resources/{rid}/notes/{nid}")))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path((rid, nid)): Path<(i64, i64)>,
) -> Result<impl IntoResponse, AppError> {
    note::delete(&pool, nid).await?;
    Ok(Redirect::to(&format!("/resources/{rid}")))
}
