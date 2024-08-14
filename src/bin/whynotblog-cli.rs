use std::{
    fs::{self},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::anyhow;
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

// Make two new commands

#[derive(Subcommand)]
enum Commands {
    /// Publishes page
    Page {
        /// The slug for the page, acts as it's id
        name: String,

        #[command(subcommand)]
        page_commands: PageCommands,
    },
    Post {
        /// The slug for the post, acts as it's id
        name: String,

        #[command(subcommand)]
        post_commands: PostCommands,
    },
}

#[derive(Subcommand)]
enum PageCommands {
    /// Create a page or update it's metadata
    Set {
        #[arg(short, long)]
        title: String,
    },
    /// Update a page's contents after it's been created
    Update {
        /// Markdown file location
        file: Option<String>,
    },
}

#[derive(Subcommand)]
enum PostCommands {
    /// Create a page or update it's metadata
    Set {
        /// Set the title, optionally creating the post
        #[arg(long)]
        title: Option<String>,

        /// Set the tag, must be done either after the post is created or at the same time
        #[arg(long)]
        tag: Option<String>,

        /// Publishes or de-publishes the post.
        #[arg(long)]
        published: Option<bool>,
    },
    /// Update a post's contents after it's been created
    Update {
        /// Markdown file location
        file: Option<String>,
    },
}

use config::{Config, Environment, File as CF, FileFormat};
use libsql::Builder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let db = Builder::new_remote(libsql_url, libsql_token)
        .build()
        .await?;

    match &cli.command {
        Commands::Page {
            name,
            page_commands,
        } => {
            match &page_commands {
                PageCommands::Set { title } => {
                    let sql = r#"INSERT INTO page (id, title) VALUES (?1, ?2);
                    ON CONFLICT(id) DO UPDATE SET title = ?2"#;

                    let conn = db.connect().unwrap();
                    conn.execute(sql, [name, title.as_str()]).await?;
                }

                PageCommands::Update { file } => {
                    let sql = "UPDATE page SET content = ?2 WHERE id = ?1;";

                    let default = format!("pages/{}.md", name);

                    let file_name = file.as_deref().unwrap_or(default.as_str());

                    let path = root.join(file_name);

                    if !path.exists() {
                        return Err(anyhow!(
                            "Path '{}' does not exist",
                            path.to_str().unwrap_or_default()
                        ));
                    }

                    let content = fs::read_to_string(path)?;

                    let parser = pulldown_cmark::Parser::new(content.as_str());

                    // Write to a new String buffer.
                    let mut html_output = String::new();
                    pulldown_cmark::html::push_html(&mut html_output, parser);

                    let conn = db.connect().unwrap();
                    conn.execute(sql, [name, html_output.as_str()]).await?;
                }
            }

            return Ok(());
        }
        Commands::Post {
            name,
            post_commands,
        } => {
            match &post_commands {
                PostCommands::Set {
                    title,
                    tag,
                    published,
                } => {
                    if let Some(title) = title {
                        let sql = r#"INSERT INTO post (slug, title) VALUES (?1, ?2)
                            ON CONFLICT(slug) DO UPDATE SET title = ?2;"#;

                        let conn = db.connect().unwrap();
                        conn.execute(sql, [name, title.as_str()]).await?;
                    }

                    if let Some(tag) = tag {
                        let sql = "UPDATE post SET tag = ?2 WHERE slug = ?1;";

                        let conn = db.connect().unwrap();
                        conn.execute(sql, [name, tag.as_str()]).await?;
                    }

                    if let Some(published) = published {
                        let sql = "UPDATE post SET published = ?2, timestamp = ?3 WHERE slug = ?1;";

                        let ts = match published {
                            true => SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            false => 0,
                        };

                        let conn = db.connect().unwrap();
                        conn.execute(sql, libsql::params![name.as_str(), published, ts])
                            .await?;
                    }
                }
                PostCommands::Update { file } => {
                    let sql = "UPDATE post SET content = ?2 WHERE slug = ?1;";

                    let default = format!("posts/{}.md", name);

                    let file_name = file.as_deref().unwrap_or(default.as_str());

                    let path = root.join(file_name);

                    if !path.exists() {
                        return Err(anyhow!(
                            "Path '{}' does not exist",
                            path.to_str().unwrap_or_default()
                        ));
                    }

                    let content = fs::read_to_string(path)?;

                    let parser = pulldown_cmark::Parser::new(content.as_str());

                    // Write to a new String buffer.
                    let mut html_output = String::new();
                    pulldown_cmark::html::push_html(&mut html_output, parser);

                    let html_output = html_output.as_str();

                    let conn = db.connect().unwrap();
                    conn.execute(sql, libsql::params![name.as_str(), html_output])
                        .await?;
                }
            }
        }
    }

    Ok(())
}
