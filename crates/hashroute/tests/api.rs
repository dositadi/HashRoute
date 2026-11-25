//! Router-level tests for the parts of the API that run before any database call:
//! the response envelope, validation, auth errors, URL checks, CORS and fallbacks.
//! The pool is lazy and points nowhere, so a test that reached the database would fail.

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use hashroute::internal::{
    AppConfig, ServerConfig,
    auth::service::jwt::JWT,
    config::{
        app_state::{AppState, Postgres},
        cors_config::CORSConfig,
        database_cfg::DatabaseCfg,
        jwt_config::JWTConfig,
    },
    platform::app::application::build_router,
};
use serde_json::{Value, json};
use tower::ServiceExt;

const ORIGIN: &str = "http://localhost:5173";

fn config() -> AppConfig {
    AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".into(),
            port: 0,
            public_base_url: "https://ho.st/".into(),
        },
        database: DatabaseCfg {
            scheme: "postgres".into(),
            password: "x".into(),
            user: "x".into(),
            db: "x".into(),
            host: "127.0.0.1".into(),
            port: 1,
            max_conn: 1,
            acquire_timeout: 1,
            idle_timeout: 1,
        },
        jwt: JWTConfig { secrets: "test-secret".into(), access_exp: 60, refresh_token_exp: 120 },
        cors: CORSConfig {
            allow_origins: ORIGIN.into(),
            allow_headers: "authorization, content-type, x-refresh".into(),
            allow_methods: "GET,POST,OPTIONS".into(),
        },
    }
}

fn app() -> Router {
    let cfg = config();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy(&cfg.database.to_dsn())
        .expect("lazy pool");
    build_router(AppState::from_pool(Postgres { db_pool: pool }, &cfg), &cfg.cors)
}

fn tokens() -> (String, String) {
    let jwt = JWT::new(config().jwt);
    let id = uuid::Uuid::new_v4();
    (jwt.access_token(id).unwrap().access_token, jwt.refresh_token(id).unwrap())
}

async fn send(req: Request<Body>) -> (StatusCode, Value) {
    let resp = app().oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() };
    (status, body)
}

fn json_req(method: &str, uri: &str) -> axum::http::request::Builder {
    Request::builder().method(method).uri(uri).header(header::CONTENT_TYPE, "application/json")
}

#[tokio::test]
async fn health_uses_data_envelope() {
    let (status, body) = send(Request::get("/").body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({ "data": { "status": "ok" } }));
}

#[tokio::test]
async fn unknown_route_is_json_404() {
    let (status, body) = send(Request::get("/tiny/v1/nope/x").body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
}

#[tokio::test]
async fn wrong_method_is_json_405() {
    let (status, body) = send(Request::delete("/tiny/v1/links").body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(body["error"]["code"], "method_not_allowed");
}

#[tokio::test]
async fn register_validation_lists_each_field() {
    let payload = json!({ "name": "a", "email": "nope", "password": "short" });
    let req =
        json_req("POST", "/tiny/v1/auth/register").body(Body::from(payload.to_string())).unwrap();
    let (status, body) = send(req).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_failed");
    let details = &body["error"]["details"];
    assert_eq!(details["name"][0], "name must be 3 to 20 characters");
    assert_eq!(details["email"][0], "email is not valid");
    assert!(!details["password"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn malformed_json_is_json_error() {
    let req = Request::post("/tiny/v1/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{not json"))
        .unwrap();
    let (status, body) = send(req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_body");
}

#[tokio::test]
async fn links_require_a_bearer_token() {
    let (status, body) = send(Request::get("/tiny/v1/links").body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "missing_token");
}

#[tokio::test]
async fn refresh_token_is_not_an_access_token() {
    let (_, refresh) = tokens();
    let req = Request::get("/tiny/v1/links")
        .header(header::AUTHORIZATION, format!("Bearer {refresh}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_token");
}

#[tokio::test]
async fn rotate_refresh_accepts_a_refresh_token() {
    let (_, refresh) = tokens();
    let req = Request::post("/tiny/v1/auth/rotate-refresh")
        .header("x-refresh", refresh)
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(req).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["data"]["access_token"].is_string());
    assert!(body["data"]["expires_at"].as_str().unwrap().contains('T'), "RFC 3339 date");
}

#[tokio::test]
async fn rotate_refresh_rejects_an_access_token() {
    let (access, _) = tokens();
    let req = Request::post("/tiny/v1/auth/rotate-refresh")
        .header("x-refresh", access)
        .body(Body::empty())
        .unwrap();
    let (status, _) = send(req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_link_rejects_unsafe_urls() {
    let (access, _) = tokens();
    for url in ["javascript:alert(1)", "data:text/html,x", "example.com"] {
        let payload = json!({ "url": url });
        let req = json_req("POST", "/tiny/v1/links")
            .header(header::AUTHORIZATION, format!("Bearer {access}"))
            .body(Body::from(payload.to_string()))
            .unwrap();
        let (status, body) = send(req).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{url}");
        assert_eq!(body["error"]["code"], "invalid_url", "{url}");
    }
}

#[tokio::test]
async fn bad_link_id_is_400() {
    let (access, _) = tokens();
    let req = Request::get("/tiny/v1/links/not-a-uuid")
        .header(header::AUTHORIZATION, format!("Bearer {access}"))
        .body(Body::empty())
        .unwrap();
    let (status, body) = send(req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_id");
}

#[tokio::test]
async fn cors_preflight_allows_configured_origin() {
    let req = Request::options("/tiny/v1/links")
        .header(header::ORIGIN, ORIGIN)
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "authorization,content-type")
        .body(Body::empty())
        .unwrap();
    let resp = app().oneshot(req).await.unwrap();
    assert_eq!(resp.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], ORIGIN);
}

#[tokio::test]
async fn cors_ignores_other_origins() {
    let req = Request::options("/tiny/v1/links")
        .header(header::ORIGIN, "https://evil.example")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .body(Body::empty())
        .unwrap();
    let resp = app().oneshot(req).await.unwrap();
    assert!(resp.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
}

#[tokio::test]
async fn cors_headers_are_on_error_responses_too() {
    // Without this the browser hides the 401 body, so the frontend can't read `token_expired`.
    let req =
        Request::get("/tiny/v1/links").header(header::ORIGIN, ORIGIN).body(Body::empty()).unwrap();
    let resp = app().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(resp.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], ORIGIN);
}
