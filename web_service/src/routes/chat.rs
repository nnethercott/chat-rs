use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use grpc_service::{Role, Turn, turn::Data};
use tonic::{Request, Streaming};
use tower_sessions::Session;
use tracing::{error, info, warn};

use crate::{Error, Result, server::AppState};

const MESSAGE_KEY: &'static str = "MESSAGES";

// GET /models/{id}/chat
pub(super) async fn chat(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    sesh: Session,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, app_state, id, sesh))
}

async fn handle_websocket(stream: WebSocket, state: AppState, _id: u32, sesh: Session) {
    // split into send and recv
    let (mut sender, mut receiver) = stream.split();

    // get convo history
    let mut messages: Vec<Turn> = sesh.get(MESSAGE_KEY).await.unwrap().unwrap_or_default();

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(query) = msg {
            info!(query=%query.as_str());

            messages.push(Turn {
                role: Role::User.into(),
                data: Some(Data::Text(query.to_string())),
            });

            let resp = get_token_stream(&state, query.to_string()).await;
            if let Ok(mut token_stream) = resp {
                // send words through ws
                while let Some(Ok(word)) = token_stream.next().await {
                    info!(token=%word);
                    if let Err(e) = sender.send(Message::Text(Utf8Bytes::from(word))).await {
                        error!(error = %e);
                    }
                }
                // send return sequence
                sender
                    .send(Message::Text(Utf8Bytes::from_static("\r\n")))
                    .await
                    .unwrap();
            } else {
                warn!(error=?resp);
                break;
            };
        }
    }

    // TODO: update messages here
    // sesh.insert(MESSAGE_KEY, value)
}

async fn get_token_stream(state: &AppState, query: String) -> Result<Streaming<String>> {
    let inference_client = state.client().ok_or(Error::UninitializedAppState)?;

    let request = Request::new(query);
    let stream = inference_client
        .lock()
        .await
        .generate_streaming(request)
        .await?;

    Ok(stream.into_inner())
}
