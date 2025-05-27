use crate::helpers::spawn_and_connect_grpc;
use axum::{
    Json,
    body::Body,
    http::{self, Request, StatusCode},
};
use clap::Parser;
use http_body_util::BodyExt;
use insta::assert_debug_snapshot;
use tower::ServiceExt;
use web_service::{
    config::{RedisConfig, Settings},
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
                .uri("/healthz")
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

    // assert 200
    assert_eq!(response.status(), StatusCode::OK);

    // assert payload
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let Json(inner) = Json::<Vec<String>>::from_bytes(&bytes).unwrap();
    assert_debug_snapshot!(inner, @r#"
    [
        "model1",
        "model2",
        "model3",
    ]
    "#);
}

#[tokio::test]
async fn test_message_extractor() {
    // TODO: check if we can use default and then update with an impl Deserialize
    let mut config: Settings = Parser::parse_from(None as Option<&str>);

    // FIXME: default isn't implemented, only default vals for clap!
    config.redis = Some(RedisConfig::default());
    dbg!("{:?}", &config.redis);

    let app = App::new_with_session_store(config)
        .await
        .unwrap()
        .into_router();

    let response = app
        .with_state(AppState::default())
        .oneshot(
            Request::builder()
                .method(http::Method::GET)
                .uri("/messages")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    //TODO: check contents
}
