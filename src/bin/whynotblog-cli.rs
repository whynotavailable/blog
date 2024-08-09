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
        /// The slug for the page, acts as it's id
        name: String,

        #[command(subcommand)]
        page_commands: PageCommands,
    },
    Post {
        name: String,
    },
    NewPost {
        #[arg(short, long)]
        title: String,

        #[arg(long)]
        tag: Option<String>,

        slug: String,
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
    Update,
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
    /// Update a page's contents after it's been created
    Update,
}

use config::{Config, Environment, File as CF, FileFormat};
use libsql::Builder;
use whynotblog::errors::AppResult;
use whynotblog::models::PostConfig;

#[tokio::main]
async fn main() -> AppResult<()> {
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
                    let sql = "INSERT OR REPLACE INTO page (id, title) VALUES (?1, ?2);";

                    let conn = db.connect().unwrap();
                    conn.execute(sql, [name, title.as_str()]).await?;
                }

                PageCommands::Update => {
                    let sql = "UPDATE page SET content = ?2 WHERE id = ?1;";

                    let path = format!("pages/{}.md", name);
                    let path = root.join(path);

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
        Commands::Post { name } => {
            let sql = r#"
                INSERT OR REPLACE INTO post
                (slug, title, timestamp, tag, content, published) VALUES
                (?1, ?2, ?3, ?4, ?5, ?6)"#;

            let data_path = format!("posts/{}.json", name);
            let data_path = root.join(data_path);

            let data_file = File::open(&data_path)?;

            let data_reader = BufReader::new(&data_file);

            let mut data: PostConfig =
                serde_json::from_reader(data_reader).expect("Failed to load data file");

            if data.published.unwrap_or(false) && data.timestamp.is_none() {
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

            let conn = db.connect().unwrap();
            conn.execute(
                sql,
                libsql::params![
                    name.as_str(),
                    data.title.as_str(),
                    data.timestamp.unwrap_or(0),
                    data.tag.as_str(),
                    html_output,
                    data.published.unwrap_or(false)
                ],
            )
            .await?;
        }
        Commands::NewPost { title, tag, slug } => {
            let data_path = format!("posts/{}.json", slug);
            let data_path = root.join(data_path);

            let data_file = File::create(&data_path)?;

            let mut data_writer = BufWriter::new(&data_file);

            serde_json::to_writer_pretty(
                &mut data_writer,
                &PostConfig {
                    title: title.clone(),
                    tag: tag.clone().unwrap_or("na".to_string()),
                    timestamp: None,
                    published: Some(false),
                },
            )?;

            data_writer.flush()?;

            let content_path = format!("posts/{}.md", slug);
            let content_path = root.join(content_path);

            let content_create = File::create_new(&content_path);

            if let Err(e) = content_create {
                println!("{}", e);
            }
        }
    }

    Ok(())
}
