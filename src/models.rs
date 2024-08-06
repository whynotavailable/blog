use std::sync::Arc;

use libsql::Database;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct RenderData {}

#[derive(Debug, Clone)]
pub struct AppState {
    pub handlebars: handlebars::Handlebars<'static>,
    pub db: Arc<Database>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RouteConfig {
    pub path: String,
    pub template: String,
    pub route_type: RouteType,
    pub page_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RouteType {
    Page,
    Post,
}

#[derive(Serialize, Debug)]
pub struct PageContent {
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct PostContent {
    pub posts: Vec<PostData>,
}

#[derive(Deserialize, Debug)]
pub struct PageData {
    pub id: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostConfig {
    pub timestamp: Option<u64>,
    pub title: String,
    pub tag: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchParams {
    pub tag: Option<String>,
    pub page: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostData {
    pub slug: String,
    pub timestamp: usize,
    pub title: String,
    pub tag: String,
    pub content: String,
}
