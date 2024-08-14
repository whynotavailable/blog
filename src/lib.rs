use std::{path::Path, sync::Arc, time::Duration};

use axum::{routing::get, Router};
use config::{Config, Environment, File, FileFormat};
use errors::AppResult;
use handlebars::{DirectorySourceOptions, Handlebars};
use libsql::Builder;
use models::AppState;

use tower_http::services::ServeDir;

pub mod data;
pub mod errors;
pub mod models;
pub mod routes;

pub async fn actual_main(root: &Path, replica: Option<String>, dev: bool) -> AppResult<()> {
    let mut handlebars = Handlebars::new();

    if dev {
        handlebars.set_dev_mode(true);
    }

    let builder = Config::builder()
        .add_source(
            File::new(
                root.join(".settings.json").to_str().unwrap(),
                FileFormat::Json,
            )
            .required(false),
        )
        .add_source(Environment::default());

    let config = builder.build()?;

    let libsql_url = config.get_string("libsql_url")?;
    let libsql_token = config.get_string("libsql_token")?;

    let db = match replica {
        Some(replica) => {
            println!("Using remote replica {}", replica);
            Builder::new_remote_replica(replica, libsql_url, libsql_token)
                .sync_interval(Duration::from_secs(300))
                .build()
                .await?
        }
        None => {
            Builder::new_remote(libsql_url, libsql_token)
                .build()
                .await?
        }
    };

    let result = handlebars
        .register_templates_directory(root.join("templates"), DirectorySourceOptions::default());

    result.unwrap();

    let state = AppState {
        handlebars,
        db: Arc::new(db),
    };

    let app = Router::new()
        .route("/", get(routes::search))
        .route("/page/:id", get(routes::handle_page))
        .route("/post/:slug", get(routes::handle_post))
        .nest_service("/assets", ServeDir::new(root.join("assets")));
    //.fallback(handler);

    //let app = app.layer(ServiceBuilder::new().layer(Extension(state)));
    let app = app.with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("WHAT LET FOOL");
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
