use std::path::Path;

use clap::{command, Parser};

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let root = cli.root.unwrap_or("./".to_string());
    let root = Path::new(root.as_str());

    whynotblog::actual_main(root, cli.dev.unwrap_or(false)).await
}

#[cfg(test)]
mod tests {
    use whynotblog::match_route;

    fn test_route_match(route: &str, format: &str) {
        let m = match_route(route, format);
        assert!(m.is_some());
    }

    fn test_route_false(route: &str, format: &str) {
        let m = match_route(route, format);
        assert!(m.is_none());
    }

    #[test]
    fn basic_tests() {
        test_route_match("/", "/");
        test_route_match("/post/asd", "/post/:slug");
        test_route_false("/post", "/admin")
    }

    #[test]
    fn extract_tests() {
        let route_match = match_route("/post/asd", "/post/:slug");

        assert!(route_match.is_some());

        assert_eq!(route_match.unwrap().get("slug").copied().unwrap(), "asd");
    }
}
