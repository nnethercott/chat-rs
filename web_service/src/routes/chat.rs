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

use crate::{Error, Result, messages::Messages, server::AppState};

// GET /models/{id}/chat
pub(super) async fn chat(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    chat_history: Messages,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, app_state, id, chat_history))
}

async fn handle_websocket(stream: WebSocket, state: AppState, _id: u32, mut messages: Messages) {
    // split into send and recv
    let (mut sender, mut receiver) = stream.split();

    // receiver implements `Stream`
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(query) = msg {
            info!(query=%query.as_str());

            // add user query to message history
            messages.push_msg(Role::Agent, query.as_str());
            messages
                .update_session()
                .await
                .map_err(|e| warn!("failed to register User message.\n{:?}", e));

            let resp = get_token_stream(&state, query.to_string()).await;
            // store agent's response so we maintain the chat history
            let mut buffer = vec![];

            if let Ok(mut token_stream) = resp {
                // send words through ws
                while let Some(Ok(word)) = token_stream.next().await {
                    info!(token=%word);
                    buffer.push(word.clone());
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

            if buffer.len() > 0 {
                messages.push_msg(Role::Agent, buffer.join(""));
                messages
                    .update_session()
                    .await
                    .map_err(|e| warn!("failed to register Agent message.\n{:?}", e));
            }
        }
    }
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
