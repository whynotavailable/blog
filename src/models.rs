use std::sync::Arc;

use libsql::Database;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct RenderData {}

#[derive(Debug, Clone)]
pub struct AppState {
    pub handlebars: handlebars::Handlebars<'static>,
    pub routes: Vec<RouteConfig>,
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

#[derive(Deserialize, Debug)]
pub struct PageData {
    pub id: String,
    pub experpt: Option<String>,
    pub content: String,
}
