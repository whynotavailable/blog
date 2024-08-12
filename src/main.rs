use std::path::Path;

use clap::{command, Parser};
use whynotblog::errors::AppResult;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Content root
    #[arg(short, long)]
    root: Option<String>,

    /// Turn on auto-reload for handlebars
    #[arg(short, long)]
    dev: Option<bool>,

    #[arg(long)]
    replica: Option<String>,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let cli = Cli::parse();
    let root = cli.root.unwrap_or("./".to_string());
    let root = Path::new(root.as_str());

    whynotblog::actual_main(root, cli.replica, cli.dev.unwrap_or(false)).await
}
