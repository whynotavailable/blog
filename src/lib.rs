use std::{collections::HashMap, fs::File, io::BufReader, path::Path, sync::Arc};

use axum::{
    extract::State,
    http::{StatusCode, Uri},
    response::Html,
    Router,
};
use config::{Config, Environment, File as CF, FileFormat};
use handlebars::{DirectorySourceOptions, Handlebars};
use libsql::{de, Builder, Connection};
use models::{AppState, PageContent, PageData, RouteConfig};
use serde::de::DeserializeOwned;
use serde_json::json;

use tower_http::services::ServeDir;

pub mod models;

async fn handler(State(state): State<Arc<AppState>>, uri: Uri) -> Result<Html<String>, StatusCode> {
    let routes: Vec<RouteConfig> = state.routes.clone();

    for route in routes {
        let route_path = route.path.clone();
        let route_data = match_route(uri.path(), route_path.as_str());

        if let Some(route_data) = route_data {
            return match route.route_type {
                models::RouteType::Page => handle_page(state, route, route_data).await,
                models::RouteType::Post => Err(StatusCode::NOT_FOUND),
            };
        }
    }

    Err(StatusCode::NOT_FOUND)
}

pub async fn get_one<T: DeserializeOwned>(
    conn: Connection,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> anyhow::Result<T> {
    let row = conn
        .query(sql, params)
        .await?
        .next()
        .await?
        .ok_or(anyhow::anyhow!("Failed to get row"))?;

    de::from_row::<T>(&row).map_err(anyhow::Error::new)
}

pub async fn get_list<T: DeserializeOwned>(
    conn: Connection,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> anyhow::Result<Vec<T>> {
    let mut iter = conn.query(sql, params).await?;
    let mut ret: Vec<T> = Vec::new();

    while let Some(page) = iter.next().await? {
        ret.push(de::from_row::<T>(&page)?);
    }

    Ok(ret)
}

pub async fn handle_page(
    state: Arc<AppState>,
    route: RouteConfig,
    _route_data: HashMap<&str, &str>,
) -> Result<Html<String>, StatusCode> {
    let conn = state
        .db
        .connect()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let content: PageData = get_one(
        conn,
        "SELECT * FROM page WHERE id = ?1",
        [route.page_id.clone()],
    )
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    let some_content = PageContent {
        content: content.content,
    };

    let html = state
        .handlebars
        .render(&route.template, &json!(some_content))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

pub async fn actual_main(root: &Path, dev: bool) -> anyhow::Result<()> {
    let mut handlebars = Handlebars::new();

    if dev {
        handlebars.set_dev_mode(true);
    }

    let builder = Config::builder()
        .add_source(
            CF::new(
                root.join(".settings.json").to_str().unwrap(),
                FileFormat::Json,
            )
            .required(false),
        )
        .add_source(Environment::default());

    let config = builder.build()?;

    let libsql_url = config.get_string("libsql_url")?;
    let libsql_token = config.get_string("libsql_token")?;

    let db = Builder::new_remote(libsql_url, libsql_token)
        .build()
        .await?;

    let result = handlebars
        .register_templates_directory(root.join("templates"), DirectorySourceOptions::default());

    result.unwrap();

    let routes_path = root.join("routes.json");

    let routes_file = File::open(&routes_path)?;

    let reader = BufReader::new(routes_file);

    let routes: Vec<RouteConfig> =
        serde_json::from_reader(reader).expect("Failed To Load Routes File!");

    let state = AppState {
        handlebars,
        routes,
        db: Arc::new(db),
    };

    let app = Router::new()
        .nest_service("/assets", ServeDir::new(root.join("assets")))
        .fallback(handler);

    //let app = app.layer(ServiceBuilder::new().layer(Extension(state)));
    let app = app.with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("WHAT LET FOOL");
    axum::serve(listener, app).await.unwrap();

    Ok(())
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
