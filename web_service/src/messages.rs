use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use crate::Result as WebResult;
use axum::{
    extract::FromRequestParts,
    response::{IntoResponseParts, ResponseParts},
};
use grpc_service::{Role, Turn, turn::Data};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

/// a wrapper around user-agent turns
#[derive(Serialize, Deserialize, Default, Debug)]
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

/// object to hold session store and update conversation history on the fly
pub struct Messages {
    pub data: MessagesData,
    pub session: Session,
}
impl Messages {
    pub const MESSAGE_KEY: &'static str = "MESSAGES";

    pub fn push_msg(&mut self, role: Role, content: impl Into<String>) {
        let turn = Turn {
            role: role.into(),
            data: Some(Data::Text(content.into())),
        };
        self.data.push(turn);
    }

    pub async fn update_session(&mut self) -> WebResult<()> {
        Ok(self.session.insert(Self::MESSAGE_KEY, &self.data).await?)
    }
}

impl Display for Messages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let contents = format!("{:?}", &self.data);
        f.write_str(&contents)
    }
}

/// implement extractor for Messages
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
