use libsql::Database;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct RenderData {}

#[derive(Debug)]
pub struct AppState {
    pub handlebars: handlebars::Handlebars<'static>,
    pub db: Database,
}

#[derive(Serialize, Debug)]
pub struct PageContent {
    pub title: Option<String>,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct PostContent {
    pub posts: Vec<PostSearchResult>,
    pub prev: Option<String>,
    pub next: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PageData {
    pub id: String,
    pub title: Option<String>,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchParams {
    pub tag: Option<String>,
    pub page: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostData {
    pub slug: String,
    pub timestamp: Option<usize>,
    pub title: String,
    pub tag: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostSearchResult {
    pub slug: String,
    pub title: String,
    pub tag: String,
}
