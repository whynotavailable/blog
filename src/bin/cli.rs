use std::path::Path;

use blog::models::PageData;
use config::{Config, Environment, File, FileFormat};
use libsql::{de, Builder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    publish().await
}

async fn _scratch() -> anyhow::Result<()> {
    let root = std::env::args().nth(1).unwrap_or("./".to_string());
    let root_path = Path::new(root.as_str());

    let builder = Config::builder()
        .add_source(Environment::default())
        .add_source(File::new(
            root_path.join(".settings.json").to_str().unwrap(),
            FileFormat::Json,
        ));

    let config = builder.build()?;

    let libsql_url = config.get_string("libsql_url")?;
    let libsql_token = config.get_string("libsql_token")?;

    let db = Builder::new_remote(libsql_url, libsql_token)
        .build()
        .await?;

    let conn = db.connect().unwrap();

    let mut pages = conn.query("SELECT * FROM page", ()).await?;

    while let Some(page) = pages.next().await? {
        let page = de::from_row::<PageData>(&page).unwrap();

        println!("{} {}", page.id, page.content);
    }

    Ok(())
}

async fn publish() -> anyhow::Result<()> {
    let sql = "INSERT OR REPLACE INTO page (id, excerpt, content) 
  VALUES (?1, ?2, ?3);";

    let page = "home";
    let content = "<h1>Hello, World!</h1>";

    let root = std::env::args().nth(1).unwrap_or("./".to_string());
    let root_path = Path::new(root.as_str());

    let builder = Config::builder()
        .add_source(Environment::default())
        .add_source(File::new(
            root_path.join(".settings.json").to_str().unwrap(),
            FileFormat::Json,
        ));

    let config = builder.build()?;

    let libsql_url = config.get_string("libsql_url")?;
    let libsql_token = config.get_string("libsql_token")?;

    let db = Builder::new_remote(libsql_url, libsql_token)
        .build()
        .await?;

    let conn = db.connect().unwrap();
    conn.execute(sql, [page, "", content]).await?;

    Ok(())
}
