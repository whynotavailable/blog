use axum::{http::Uri, response::Html, routing::get, Router};

async fn handler(uri: Uri) -> Html<String> {
    Html(uri.to_string())
}

#[tokio::main]
async fn main() {
    println!("hi");
    let app = Router::new()
        .route("/", get(handler))
        .route("/*e", get(handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
