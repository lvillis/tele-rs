use serde_json::json;

use super::*;

/// Builds a synthetic message update.
pub fn message_update(update_id: i64, chat_id: i64, text: &str) -> Result<Update> {
    serde_json::from_value(json!({
        "update_id": update_id,
        "message": {
            "message_id": update_id,
            "date": 1700000000 + update_id,
            "chat": {"id": chat_id, "type": "private"},
            "text": text
        }
    }))
    .map_err(|source| invalid_request(format!("failed to build message update fixture: {source}")))
}

/// Builds a synthetic callback update.
pub fn callback_update(update_id: i64, chat_id: i64, data: &str) -> Result<Update> {
    serde_json::from_value(json!({
        "update_id": update_id,
        "callback_query": {
            "id": format!("cb-{update_id}"),
            "from": {
                "id": 123,
                "is_bot": false,
                "first_name": "tester"
            },
            "message": {
                "message_id": update_id,
                "date": 1700000000 + update_id,
                "chat": {"id": chat_id, "type": "private"},
                "text": "button clicked"
            },
            "data": data
        }
    }))
    .map_err(|source| invalid_request(format!("failed to build callback update fixture: {source}")))
}

/// Lightweight router harness for fast bot handler tests.
pub struct BotHarness {
    client: Client,
    router: Router,
}

impl BotHarness {
    pub fn new(router: Router) -> Result<Self> {
        let client = Client::builder("http://127.0.0.1:9")?
            .bot_token("123:abc")?
            .build()?;
        Ok(Self::with_client(client, router))
    }

    pub fn with_client(client: Client, router: Router) -> Self {
        Self { client, router }
    }

    pub fn dispatch(
        &self,
        update: Update,
    ) -> Pin<Box<dyn Future<Output = Result<DispatchOutcome>> + Send + '_>> {
        Box::pin(async move {
            let update_id = update.update_id;
            let context = BotContext::new(self.client.clone());
            match self.router.dispatch_prepared(context, update).await? {
                true => Ok(DispatchOutcome::Handled { update_id }),
                false => Ok(DispatchOutcome::Ignored { update_id }),
            }
        })
    }

    pub fn dispatch_json(
        &self,
        payload: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<DispatchOutcome>> + Send + '_>> {
        let update: Result<Update> = serde_json::from_slice(payload).map_err(|source| {
            invalid_request(format!(
                "failed to deserialize test update payload: {source}"
            ))
        });
        Box::pin(async move { self.dispatch(update?).await })
    }
}
