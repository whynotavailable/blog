use std::collections::HashMap;

pub fn match_route<'a>(route: &'a str, format: &'a str) -> Option<HashMap<&'a str, &'a str>> {
    let mut param_map: HashMap<&str, &str> = HashMap::new();

    let route_parts: Vec<&str> = route.trim_start_matches("/").split("/").collect();
    let format_parts: Vec<&str> = format.trim_start_matches("/").split("/").collect();

    if route_parts.len() != format_parts.len() {
        return None;
    }

    let my_range = 0..route_parts.len();

    for i in my_range {
        let route_part = route_parts[i];
        let format_part = format_parts[i];

        if format_part.starts_with(":") {
            // do
            let key = format_part.trim_start_matches(":");
            param_map.insert(key, route_part);
        } else if !route_part.eq(format_part) {
            return None;
        }
    }

    Some(param_map)
}

