use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::is_false;

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ForumTopicEdited {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_custom_emoji_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ForumTopicClosed {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ForumTopicReopened {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GeneralForumTopicHidden {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct GeneralForumTopicUnhidden {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
