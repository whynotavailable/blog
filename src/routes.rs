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
    models::{
        AppState, PageContent, PageData, PostContent, PostData, PostSearchResult, SearchParams,
    },
};

pub async fn handle_page(
    State(state): State<Arc<AppState>>,
    extract::Path(id): extract::Path<String>,
) -> AppResult<Html<String>> {
    let conn = state.db.connect()?;

    let content: PageData = data::get_one(conn, "SELECT * FROM page WHERE id = ?1", [id])
        .await
        .map_err(|_| AppError::status(StatusCode::NOT_FOUND))?;

    let some_content = PageContent {
        title: content.title,
        content: content.content,
    };

    let html = state.handlebars.render("page", &json!(some_content))?;

    Ok(Html(html))
}

pub async fn handle_post(
    State(state): State<Arc<AppState>>,
    extract::Path(slug): extract::Path<String>,
) -> AppResult<Html<String>> {
    let conn = state.db.connect()?;

    let content: PostData = data::get_one(conn, "SELECT * FROM post WHERE slug = ?1", [slug])
        .await
        .map_err(|_| AppError::status(StatusCode::NOT_FOUND))?;

    let html = state.handlebars.render("post", &json!(content))?;

    Ok(Html(html))
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    search_params: Query<SearchParams>,
) -> AppResult<Html<String>> {
    let conn = state.db.connect()?;

    let mut prev: Option<String> = None;
    let mut next: Option<String> = None;

    let skip: u32 = search_params.page.unwrap_or(0) * 8;
    let posts: Vec<PostSearchResult> = match search_params.tag.clone() {
        Some(tag) => {
            let sql = "SELECT slug, tag, title FROM post WHERE published = TRUE AND tag = ?1 ORDER BY timestamp DESC LIMIT 9 OFFSET ?2";

            data::get_list(conn, sql, libsql::params![tag.as_str(), skip]).await?
        }
        None => {
            let sql = "SELECT slug, tag, title FROM post WHERE published = TRUE ORDER BY timestamp DESC LIMIT 9 OFFSET ?1";

            data::get_list(conn, sql, libsql::params![skip]).await?
        }
    };

    match search_params.tag.clone() {
        Some(tag) => {
            if search_params.page.is_some() {
                let page = search_params.page.unwrap_or(0);

                prev = match page {
                    1 => Some(format!("/?tag={}", tag)),
                    n if n > 1 => Some(format!("/?tag={}&page={}", tag, page - 1)),
                    _ => None,
                };

                if posts.len() == 9 {
                    next = Some(format!("/?tag={}&page={}", tag, page + 1));
                }
            }
        }
        None => {
            if search_params.page.is_some() {
                let page = search_params.page.unwrap_or(0);

                prev = match page {
                    1 => Some("/".to_string()),
                    n if n > 1 => Some(format!("/?page={}", page - 1)),
                    _ => None,
                };

                if posts.len() == 9 {
                    next = Some(format!("/?page={}", page + 1));
                }
            }
        }
    };

    let content = PostContent { posts, prev, next };

    let html = state.handlebars.render("search", &json!(content))?;

    Ok(Html(html))
}
