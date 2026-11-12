use std::str::FromStr;

use axum::http::{HeaderName, HeaderValue, Method, method};
use tower_http::cors::{CorsLayer};

use crate::internal::config::cors_config::CORSConfig;

pub fn cors(cfg: &CORSConfig) -> CorsLayer {
    let allowed_headers = cfg
        .allow_headers
        .split(",")
        .map(|h| {
            HeaderName::from_str(h).expect(format!("invalid header: {h}").as_str())
        }).collect::<Vec<HeaderName>>();

    let allowed_methods = cfg
        .allow_methods
        .split(",")
        .map(|m| {
            method::Method::from_str(m).expect(format!("invalid method: {m}").as_str())
        }).collect::<Vec<Method>>();

    let allowed_origin = cfg
        .allow_headers
        .split(",")
        .map(|o| {
            o.parse().expect(format!("invalid origin, must not have a trailing slash: {o}").as_str())
        }).collect::<Vec<HeaderValue>>();

    CorsLayer::new().allow_headers(allowed_headers).allow_methods(allowed_methods).allow_origin(allowed_origin).allow_credentials(true)
}