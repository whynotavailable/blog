use std::{borrow::Borrow, collections::HashMap, sync::Arc};

use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::Html,
    routing::get,
    Extension, Router,
};
use handlebars::{DirectorySourceOptions, Handlebars};
use models::{AppState, RouteConfig};
use serde_json::json;
use tower::ServiceBuilder;

pub mod models;

async fn handler(State(state): State<Arc<AppState>>, uri: Uri) -> Result<Html<String>, StatusCode> {
    let routes: Vec<RouteConfig> = state.routes.clone();

    for route in routes {
        let route_data = match_route(uri.path(), &route.path);

        if let Some(_route_data) = route_data {
            let html = state
                .handlebars
                .render(&route.template, &json!(()))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            return Ok(Html(html));
        }
    }

    Err(StatusCode::NOT_FOUND)
}

pub async fn actual_main() {
    let mut handlebars = Handlebars::new();

    let result =
        handlebars.register_templates_directory("templates/", DirectorySourceOptions::default());

    result.unwrap();

    let mut routes: Vec<RouteConfig> = Vec::new();

    routes.push(RouteConfig {
        path: "".to_string(),
        template: "base".to_string(),
        route_type: models::RouteType::Page,
    });

    let state = AppState { handlebars, routes };

    let app = Router::new()
        .route("/", get(handler))
        .route("/*e", get(handler));

    //let app = app.layer(ServiceBuilder::new().layer(Extension(state)));
    let app = app.with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub fn match_route<'a>(route: &'a str, format: &'a str) -> Option<HashMap<&'a str, &'a str>> {
    let mut param_map: HashMap<&str, &str> = HashMap::new();

    let route_parts: Vec<&str> = route.trim_start_matches("/").split("/").collect();
    let format_parts: Vec<&str> = format.trim_start_matches("/").split("/").collect();

    if route_parts.len() != format_parts.len() {
        return None;
    }

    let my_range = 0..route_parts.len();

    for i in my_range {
        let route_part = route_parts[i];
        let format_part = format_parts[i];

        if format_part.starts_with(":") {
            // do
            let key = format_part.trim_start_matches(":");
            param_map.insert(key, route_part);
        } else if !route_part.eq(format_part) {
            return None;
        }
    }

    Some(param_map)
}
