use axum::{
    Router,
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use insta::{assert_debug_snapshot, assert_snapshot};
use tower::ServiceExt;
use web_service::{
    config::Settings,
    server::{App, AppState},
};

// a test not using axum-test
#[tokio::test]
async fn test_health() {
    let app = App::new(Settings::default()).unwrap().into_router();

    let response = app
        .with_state(AppState::default())
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // empty body but adding this as a reference for later ...
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_debug_snapshot!(bytes);
}
