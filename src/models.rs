use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct RenderData {}

#[derive(Debug, Clone)]
pub struct AppState {
    pub handlebars: handlebars::Handlebars<'static>,
    pub routes: Vec<RouteConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RouteConfig {
    pub path: String,
    pub template: String,
    pub route_type: RouteType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RouteType {
    Page,
    Post,
}
