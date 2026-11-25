use axum::http::{HeaderName, HeaderValue, Method};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::internal::config::cors_config::CORSConfig;

fn split(list: &str) -> impl Iterator<Item = &str> {
    list.split(',').map(str::trim).filter(|s| !s.is_empty())
}

/// Builds the CORS layer. Panics at startup on a malformed entry so a bad config fails loudly
/// instead of silently blocking the frontend.
pub fn cors(cfg: &CORSConfig) -> CorsLayer {
    let headers: Vec<HeaderName> = split(&cfg.allow_headers)
        .map(|h| h.parse().unwrap_or_else(|_| panic!("ALLOW_HEADERS: invalid header {h:?}")))
        .collect();

    let methods: Vec<Method> = split(&cfg.allow_methods)
        .map(|m| m.parse().unwrap_or_else(|_| panic!("ALLOW_METHODS: invalid method {m:?}")))
        .collect();

    let layer = CorsLayer::new().allow_headers(headers).allow_methods(methods);

    // Auth uses the Authorization / X-Refresh headers, not cookies, so a wildcard is safe.
    if cfg.allow_origins.trim() == "*" {
        return layer.allow_origin(Any);
    }

    let origins: Vec<HeaderValue> = split(&cfg.allow_origins).map(parse_origin).collect();

    layer.allow_origin(AllowOrigin::list(origins))
}

fn parse_origin(origin: &str) -> HeaderValue {
    let valid = (origin.starts_with("http://") || origin.starts_with("https://"))
        && !origin.ends_with('/');
    assert!(
        valid,
        "ALLOW_ORIGINS: {origin:?} must look like http://host[:port] (with a scheme, no trailing slash)"
    );

    origin.parse().unwrap_or_else(|_| panic!("ALLOW_ORIGINS: invalid origin {origin:?}"))
}
