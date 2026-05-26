use std::collections::BTreeMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::extra::{
    field_len as extra_field_len, serialize_fields as serialize_extra_fields,
    serialize_optional_field,
};

fn serialize_extra_only<S>(
    serializer: S,
    extra: &BTreeMap<String, Value>,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut object = serializer.serialize_map(Some(extra.len()))?;
    serialize_extra_fields(&mut object, extra, &[])?;
    object.end()
}

/// Telegram forum topic object.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ForumTopic {
    pub message_thread_id: i64,
    pub name: String,
    pub icon_color: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_custom_emoji_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ForumTopic {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "message_thread_id",
            "name",
            "icon_color",
            "icon_custom_emoji_id",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.icon_custom_emoji_id.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 3))?;
        object.serialize_entry("message_thread_id", &self.message_thread_id)?;
        object.serialize_entry("name", &self.name)?;
        object.serialize_entry("icon_color", &self.icon_color)?;
        serialize_optional_field(
            &mut object,
            "icon_custom_emoji_id",
            &self.icon_custom_emoji_id,
        )?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ForumTopicCreated {
    pub name: String,
    pub icon_color: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_custom_emoji_id: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_name_implicit: bool,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ForumTopicCreated {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "name",
            "icon_color",
            "icon_custom_emoji_id",
            "is_name_implicit",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len =
            usize::from(self.icon_custom_emoji_id.is_some()) + usize::from(self.is_name_implicit);
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 2))?;
        object.serialize_entry("name", &self.name)?;
        object.serialize_entry("icon_color", &self.icon_color)?;
        serialize_optional_field(
            &mut object,
            "icon_custom_emoji_id",
            &self.icon_custom_emoji_id,
        )?;
        if self.is_name_implicit {
            object.serialize_entry("is_name_implicit", &self.is_name_implicit)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ForumTopicEdited {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_custom_emoji_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ForumTopicEdited {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["name", "icon_custom_emoji_id"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len =
            usize::from(self.name.is_some()) + usize::from(self.icon_custom_emoji_id.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len))?;
        serialize_optional_field(&mut object, "name", &self.name)?;
        serialize_optional_field(
            &mut object,
            "icon_custom_emoji_id",
            &self.icon_custom_emoji_id,
        )?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct ForumTopicClosed {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ForumTopicClosed {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize_extra_only(serializer, &self.extra)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct ForumTopicReopened {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ForumTopicReopened {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize_extra_only(serializer, &self.extra)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct GeneralForumTopicHidden {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for GeneralForumTopicHidden {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize_extra_only(serializer, &self.extra)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[non_exhaustive]
pub struct GeneralForumTopicUnhidden {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for GeneralForumTopicUnhidden {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize_extra_only(serializer, &self.extra)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ForumTopic, ForumTopicClosed, ForumTopicCreated, ForumTopicEdited, GeneralForumTopicHidden,
    };

    #[test]
    fn forum_topic_extra_cannot_override_reserved_fields() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut topic: ForumTopic = serde_json::from_value(json!({
            "message_thread_id": 11,
            "name": "Announcements",
            "icon_color": 7322096,
            "future": {"kept": true}
        }))?;
        topic.extra.insert("message_thread_id".to_owned(), json!(1));
        topic.extra.insert("name".to_owned(), json!("spoofed"));
        topic.extra.insert("icon_color".to_owned(), json!(0));
        topic
            .extra
            .insert("icon_custom_emoji_id".to_owned(), json!("spoofed"));
        topic
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let value = serde_json::to_value(topic)?;
        assert_eq!(value["message_thread_id"], 11);
        assert_eq!(value["name"], "Announcements");
        assert_eq!(value["icon_color"], 7322096);
        assert!(value.get("icon_custom_emoji_id").is_none());
        assert_eq!(value["future"], json!({"kept": true}));
        assert_eq!(value["another_future"], "kept");
        Ok(())
    }

    #[test]
    fn forum_topic_event_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut created: ForumTopicCreated = serde_json::from_value(json!({
            "name": "Announcements",
            "icon_color": 7322096,
            "future": {"kept": true}
        }))?;
        created.extra.insert("name".to_owned(), json!("spoofed"));
        created.extra.insert("icon_color".to_owned(), json!(0));
        created
            .extra
            .insert("is_name_implicit".to_owned(), json!(true));
        created
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let created_value = serde_json::to_value(created)?;
        assert_eq!(created_value["name"], "Announcements");
        assert_eq!(created_value["icon_color"], 7322096);
        assert!(created_value.get("is_name_implicit").is_none());
        assert_eq!(created_value["future"], json!({"kept": true}));
        assert_eq!(created_value["another_future"], "kept");

        let mut edited: ForumTopicEdited = serde_json::from_value(json!({
            "name": "Updated",
            "future": {"kept": true}
        }))?;
        edited.extra.insert("name".to_owned(), json!("spoofed"));
        edited
            .extra
            .insert("icon_custom_emoji_id".to_owned(), json!("spoofed"));
        edited
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let edited_value = serde_json::to_value(edited)?;
        assert_eq!(edited_value["name"], "Updated");
        assert!(edited_value.get("icon_custom_emoji_id").is_none());
        assert_eq!(edited_value["future"], json!({"kept": true}));
        assert_eq!(edited_value["another_future"], "kept");
        Ok(())
    }

    #[test]
    fn empty_forum_events_preserve_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let closed: ForumTopicClosed = serde_json::from_value(json!({
            "future": {"kept": true}
        }))?;
        assert_eq!(
            serde_json::to_value(closed)?,
            json!({"future": {"kept": true}})
        );

        let hidden: GeneralForumTopicHidden = serde_json::from_value(json!({
            "future": {"kept": true}
        }))?;
        assert_eq!(
            serde_json::to_value(hidden)?,
            json!({"future": {"kept": true}})
        );
        Ok(())
    }
}
