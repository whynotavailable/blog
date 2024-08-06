use std::sync::Arc;

use axum::{
    extract::{self, Query, State},
    http::StatusCode,
    response::Html,
};
use serde_json::json;

use crate::{
    data,
    errors::{AppError, AppResult},
    models::{AppState, PageContent, PageData, PostContent, PostData, SearchParams},
};

pub async fn handle_page(
    State(state): State<Arc<AppState>>,
    extract::Path(id): extract::Path<String>,
) -> AppResult<Html<String>> {
    let conn = state.db.connect().map_err(AppError::from)?;

    let content: PageData = data::get_one(conn, "SELECT * FROM page WHERE id = ?1", [id])
        .await
        .map_err(|_| AppError::status(StatusCode::NOT_FOUND))?;

    let some_content = PageContent {
        content: content.content,
    };

    let html = state
        .handlebars
        .render("page", &json!(some_content))
        .map_err(AppError::from)?;

    Ok(Html(html))
}

pub async fn handle_post(
    State(state): State<Arc<AppState>>,
    extract::Path(slug): extract::Path<String>,
) -> AppResult<Html<String>> {
    let conn = state.db.connect().map_err(AppError::from)?;

    let content: PostData = data::get_one(conn, "SELECT * FROM post WHERE slug = ?1", [slug])
        .await
        .map_err(|_| AppError::status(StatusCode::NOT_FOUND))?;

    let html = state
        .handlebars
        .render("post", &json!(content))
        .map_err(AppError::from)?;

    Ok(Html(html))
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    search_params: Query<SearchParams>,
) -> AppResult<Html<String>> {
    let conn = state.db.connect().map_err(AppError::from)?;

    let skip: u32 = search_params.page.unwrap_or(0) * 8;
    let posts: Vec<PostData> = match search_params.tag.clone() {
        Some(tag) => {
            let sql = "SELECT * FROM post WHERE tag = ?1 ORDER BY timestamp DESC LIMIT 9 OFFSET ?2";

            data::get_list(conn, sql, libsql::params![tag.as_str(), skip]).await?
        }
        None => {
            let sql = "SELECT * FROM post ORDER BY timestamp DESC LIMIT 9 OFFSET ?1";

            data::get_list(conn, sql, libsql::params![skip]).await?
        }
    };

    let content = PostContent { posts };

    let html = state
        .handlebars
        .render("search", &json!(content))
        .map_err(AppError::from)?;

    Ok(Html(html))
}
