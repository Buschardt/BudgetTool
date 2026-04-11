use axum::body::Body;
use axum::http::{Request, StatusCode};
use budgettool_api::db;
use budgettool_api::models::AppState;
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn test_state() -> AppState {
    let pool = db::init_pool("sqlite::memory:").await;
    let hash = bcrypt::hash("testpass", 4).unwrap(); // low cost for fast tests
    sqlx::query("INSERT INTO users (username, password) VALUES (?, ?)")
        .bind("alice")
        .bind(&hash)
        .execute(&pool)
        .await
        .unwrap();
    AppState {
        db: pool,
        jwt_secret: "test-secret".into(),
        data_dir: std::path::PathBuf::from("/tmp/budgettool-test"),
    }
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn login_request(username: &str, password: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/login")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::json!({ "username": username, "password": password }).to_string(),
        ))
        .unwrap()
}

#[tokio::test]
async fn health_is_public() {
    let state = test_state().await;
    let resp = budgettool_api::app(state)
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["ok"], true);
}

#[tokio::test]
async fn login_with_valid_credentials() {
    let state = test_state().await;
    let resp = budgettool_api::app(state)
        .oneshot(login_request("alice", "testpass"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["ok"], true);
    assert!(json["data"]["token"].as_str().is_some());
}

#[tokio::test]
async fn login_with_wrong_password() {
    let state = test_state().await;
    let resp = budgettool_api::app(state)
        .oneshot(login_request("alice", "wrongpass"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_with_nonexistent_user() {
    let state = test_state().await;
    let resp = budgettool_api::app(state)
        .oneshot(login_request("bob", "testpass"))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_without_token_is_401() {
    let state = test_state().await;
    let resp = budgettool_api::app(state)
        .oneshot(
            Request::builder()
                .uri("/api/me")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_with_valid_token() {
    let state = test_state().await;
    // Login first
    let login_resp = budgettool_api::app(state.clone())
        .oneshot(login_request("alice", "testpass"))
        .await
        .unwrap();
    let login_json = body_json(login_resp).await;
    let token = login_json["data"]["token"].as_str().unwrap();

    // Use token on /api/me
    let resp = budgettool_api::app(state)
        .oneshot(
            Request::builder()
                .uri("/api/me")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = body_json(resp).await;
    assert_eq!(json["ok"], true);
    assert_eq!(json["data"]["username"], "alice");
}
