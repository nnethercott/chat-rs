use std::ops::{Deref, DerefMut};

use crate::Result as WebResult;
use axum::{
    extract::FromRequestParts,
    response::{IntoResponseParts, ResponseParts},
};
use grpc_service::Turn;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MessagesData(Vec<Turn>);

impl Deref for MessagesData {
    type Target = Vec<Turn>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for MessagesData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Messages {
    data: MessagesData,
    session: Session,
}
impl Messages {
    const MESSAGE_KEY: &'static str = "MESSAGES";

    pub fn push(&mut self, turn: Turn){
        self.data.push(turn);
    }

    pub async fn update_session(self, turn: Turn) -> WebResult<()> {
        let mut data = self.data;
        data.push(turn);
        Ok(self.session.insert(Self::MESSAGE_KEY, data).await?)
    }
}

impl<S> FromRequestParts<S> for Messages
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        let data: MessagesData = session
            .get(Self::MESSAGE_KEY)
            .await
            .unwrap()
            .unwrap_or_default();
        Ok(Self { data, session })
    }
}
