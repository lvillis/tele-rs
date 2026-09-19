use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]

pub struct EphemeralMessageParameters {
    pub receiver_user_id: crate::types::common::UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback_query_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replace_callback_query_message: Option<bool>,
}

impl EphemeralMessageParameters {
    pub fn new(receiver_user_id: crate::types::common::UserId) -> Self {
        Self {
            receiver_user_id,
            callback_query_id: None,
            replace_callback_query_message: None,
        }
    }
    pub fn validate(&self) -> crate::Result<()> {
        self.receiver_user_id.validate()?;
        if let Some(id) = &self.callback_query_id {
            crate::types::validation::string_id("callback_query_id", id)?;
        }
        if self.replace_callback_query_message == Some(true) && self.callback_query_id.is_none() {
            return Err(crate::Error::InvalidRequest {
                reason: "replacing a callback message requires callback_query_id".to_owned(),
            });
        }
        Ok(())
    }
}
