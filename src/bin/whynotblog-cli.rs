use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Content root
    #[arg(short, long)]
    root: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Publishes page
    Page {
        name: String,
    },
    Post {
        name: String,
    },
}

use config::{Config, Environment, File as CF, FileFormat};
use libsql::Builder;
use whynotblog::models::PostConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    publish().await
}

async fn publish() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let root = cli.root.unwrap_or("./".to_string());
    let root = Path::new(root.as_str());

    let builder = Config::builder()
        .add_source(Environment::default())
        .add_source(CF::new(
            root.join(".settings.json").to_str().unwrap(),
            FileFormat::Json,
        ));

    let config = builder.build()?;

    let libsql_url = config.get_string("libsql_url")?;
    let libsql_token = config.get_string("libsql_token")?;

    match &cli.command {
        Commands::Page { name } => {
            let sql = "INSERT OR REPLACE INTO page (id, content) VALUES (?1, ?2);";

            let path = format!("pages/{}.md", name);
            let path = root.join(path);

            let content = fs::read_to_string(path)?;

            let parser = pulldown_cmark::Parser::new(content.as_str());

            // Write to a new String buffer.
            let mut html_output = String::new();
            pulldown_cmark::html::push_html(&mut html_output, parser);

            let db = Builder::new_remote(libsql_url, libsql_token)
                .build()
                .await?;

            let conn = db.connect().unwrap();
            conn.execute(sql, [name, html_output.as_str()]).await?;
        }
        Commands::Post { name } => {
            let sql = r#"
                INSERT OR REPLACE INTO post
                (slug, title, timestamp, tag, content) VALUES
                (?1, ?2, ?3, ?4, ?5)"#;

            let data_path = format!("posts/{}.json", name);
            let data_path = root.join(data_path);

            let data_file = File::open(&data_path)?;

            let data_reader = BufReader::new(&data_file);

            let mut data: PostConfig =
                serde_json::from_reader(data_reader).expect("Failed to load data file");

            if data.timestamp.is_none() {
                let data_file = File::options().write(true).open(&data_path)?;

                data.timestamp = Some(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );

                let mut data_writer = BufWriter::new(&data_file);

                serde_json::to_writer_pretty(&mut data_writer, &data)?;

                data_writer.flush()?;
            }

            let content_path = format!("posts/{}.md", name);
            let content_path = root.join(content_path);

            let content = fs::read_to_string(content_path)?;

            let parser = pulldown_cmark::Parser::new(content.as_str());

            // Write to a new String buffer.
            let mut html_output = String::new();
            pulldown_cmark::html::push_html(&mut html_output, parser);

            let html_output = html_output.as_str();

            let db = Builder::new_remote(libsql_url, libsql_token)
                .build()
                .await?;

            let conn = db.connect().unwrap();
            conn.execute(
                sql,
                libsql::params![
                    name.as_str(),
                    data.title.as_str(),
                    data.timestamp
                        .expect("Require Timestamp (should never happen)"),
                    data.tag.as_str(),
                    html_output,
                ],
            )
            .await?;
        }
    }

    Ok(())
}
