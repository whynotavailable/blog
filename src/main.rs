#[tokio::main]
async fn main() {
    let root = std::env::args().nth(1).unwrap_or("./".to_string());

    blog::actual_main(root).await;
}

#[cfg(test)]
mod tests {
    use blog::match_route;

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
