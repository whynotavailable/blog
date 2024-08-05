use std::{path::Path, sync::Arc};

use axum::{
    extract::{self, Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use config::{Config, Environment, File as CF, FileFormat};
use handlebars::{DirectorySourceOptions, Handlebars};
use libsql::{de, Builder, Connection};
use models::{AppState, PageContent, PageData, PostContent, PostData, SearchParams};
use serde::de::DeserializeOwned;
use serde_json::json;

use tower_http::services::ServeDir;

pub mod models;

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
    State(state): State<Arc<AppState>>,
    extract::Path(id): extract::Path<String>,
) -> Result<Html<String>, StatusCode> {
    let conn = state
        .db
        .connect()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let content: PageData = get_one(conn, "SELECT * FROM page WHERE id = ?1", [id])
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let some_content = PageContent {
        content: content.content,
    };

    let html = state
        .handlebars
        .render("page", &json!(some_content))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

pub async fn handle_post(
    State(state): State<Arc<AppState>>,
    extract::Path(slug): extract::Path<String>,
) -> Result<Html<String>, StatusCode> {
    let conn = state
        .db
        .connect()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let content: PostData = get_one(conn, "SELECT * FROM post WHERE slug = ?1", [slug])
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let html = state
        .handlebars
        .render("post", &json!(content))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    search_params: Query<SearchParams>,
) -> Result<Html<String>, StatusCode> {
    let conn = state
        .db
        .connect()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let posts: Vec<PostData> = match search_params.tag.clone() {
        Some(tag) => {
            let sql = "SELECT * FROM post WHERE tag = ?1 ORDER BY timestamp DESC";

            get_list(conn, sql, libsql::params![tag.as_str()])
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
        None => {
            let sql = "SELECT * FROM post ORDER BY timestamp DESC";

            get_list(conn, sql, ())
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    let content = PostContent { posts };

    let html = state
        .handlebars
        .render("search", &json!(content))
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

    let state = AppState {
        handlebars,
        db: Arc::new(db),
    };

    let app = Router::new()
        .route("/", get(search))
        .route("/page/:id", get(handle_page))
        .route("/post/:slug", get(handle_post))
        .nest_service("/assets", ServeDir::new(root.join("assets")));
    //.fallback(handler);

    //let app = app.layer(ServiceBuilder::new().layer(Extension(state)));
    let app = app.with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("WHAT LET FOOL");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
