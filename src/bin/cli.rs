use std::{fs, path::Path};

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
    Page { name: String },
}

use config::{Config, Environment, File, FileFormat};
use libsql::Builder;

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
        .add_source(File::new(
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
    }

    Ok(())
}
