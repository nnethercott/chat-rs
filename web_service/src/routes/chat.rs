use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use tonic::{Request, Streaming};
use tracing::{info, warn};

use crate::{Error, Result, server::AppState};

// GET /models/{id}/chat
pub(super) async fn chat(
    ws: WebSocketUpgrade,
    State(app_state): State<AppState>,
    Path(id): Path<u32>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, app_state, id))
}

async fn handle_websocket(mut stream: WebSocket, state: AppState, id: u32) {
    // split into send and recv
    let (mut sender, mut receiver) = stream.split();

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(query) = msg {
            info!(query=%query.as_str());

            let resp = get_token_stream(&state, query.to_string()).await;
            if let Ok(mut token_stream) = resp {
                // send words through ws
                while let Some(Ok(word)) = token_stream.next().await {
                    info!(token=%word);
                    sender.send(Message::Text(Utf8Bytes::from(word))).await;
                }
                // send return sequence ?
                sender.send(Message::Text(Utf8Bytes::from_static("\r\n"))).await;
            } else {
                warn!(error=?resp);
                break;
            };
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
