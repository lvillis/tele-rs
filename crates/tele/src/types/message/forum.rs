use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

/// Telegram forum topic object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ForumTopic {
    pub message_thread_id: i64,
    pub name: String,
    pub icon_color: u32,
    #[serde(default)]
    pub icon_custom_emoji_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ForumTopicCreated {
    pub name: String,
    pub icon_color: u32,
    #[serde(default)]
    pub icon_custom_emoji_id: Option<String>,
    #[serde(default)]
    pub is_name_implicit: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ForumTopicEdited {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub icon_custom_emoji_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct ForumTopicClosed {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct ForumTopicReopened {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct GeneralForumTopicHidden {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct GeneralForumTopicUnhidden {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ForumTopic, ForumTopicClosed, ForumTopicCreated, ForumTopicEdited, GeneralForumTopicHidden,
    };

    #[test]
    fn forum_topic_preserves_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let topic: ForumTopic = serde_json::from_value(json!({
            "message_thread_id": 11,
            "name": "Announcements",
            "icon_color": 7322096,
            "future": {"kept": true}
        }))?;

        assert_eq!(topic.message_thread_id, 11);
        assert_eq!(topic.name, "Announcements");
        assert_eq!(topic.icon_color, 7322096);
        assert_eq!(topic.extra.get("future"), Some(&json!({"kept": true})));
        Ok(())
    }

    #[test]
    fn forum_topic_events_preserve_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let created: ForumTopicCreated = serde_json::from_value(json!({
            "name": "Announcements",
            "icon_color": 7322096,
            "future": {"kept": true}
        }))?;
        assert_eq!(created.name, "Announcements");
        assert_eq!(created.icon_color, 7322096);
        assert_eq!(created.extra.get("future"), Some(&json!({"kept": true})));

        let edited: ForumTopicEdited = serde_json::from_value(json!({
            "name": "Updated",
            "future": {"kept": true}
        }))?;
        assert_eq!(edited.name.as_deref(), Some("Updated"));
        assert_eq!(edited.extra.get("future"), Some(&json!({"kept": true})));
        Ok(())
    }

    #[test]
    fn empty_forum_events_preserve_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let closed: ForumTopicClosed = serde_json::from_value(json!({
            "future": {"kept": true}
        }))?;
        assert_eq!(closed.extra.get("future"), Some(&json!({"kept": true})));

        let hidden: GeneralForumTopicHidden = serde_json::from_value(json!({
            "future": {"kept": true}
        }))?;
        assert_eq!(hidden.extra.get("future"), Some(&json!({"kept": true})));
        Ok(())
    }
}
