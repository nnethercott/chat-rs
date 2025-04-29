use std::io::Read;

use axum::{
    Router,
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use insta::{assert_binary_snapshot, assert_debug_snapshot, assert_snapshot, assert_yaml_snapshot};
use tower::ServiceExt;
use web_service::{
    config::Settings,
    server::{App, AppState},
};

use crate::helpers::spawn_and_connect_grpc;

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
}

#[tokio::test]
async fn test_list_models_endpoint() {
    // init mock grpc
    let client = spawn_and_connect_grpc().await;

    // setup web
    let app = App::new(Settings::default()).unwrap().into_router();
    let state = AppState::new(client);

    let response = app
        .with_state(state)
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/models/list")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // empty body but adding this as a reference for later ...
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    dbg!(&bytes);
    assert_debug_snapshot!(bytes, @r#"b"[\"model\"]""#);

}
