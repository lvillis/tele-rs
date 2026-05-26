use std::collections::BTreeMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::{ChatId, MessageId, UserId};
use crate::types::extra::{
    field_len as extra_field_len, serialize_fields as serialize_extra_fields,
};
use crate::types::tagged::{serialize_tagged_field, strip_tag, tagged_field};
use crate::{Error, Result};

const MAX_CHAT_TITLE_CHARS: usize = 128;
const MAX_CHAT_DESCRIPTION_CHARS: usize = 255;
const MAX_CUSTOM_TITLE_CHARS: usize = 16;
const MAX_INVITE_LINK_NAME_CHARS: usize = 32;
const MAX_INVITE_LINK_MEMBER_LIMIT: u32 = 99_999;
const MAX_STICKER_SET_NAME_CHARS: usize = 64;

/// Telegram chat permissions object.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatPermissions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_send_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_send_audios: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_send_documents: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_send_photos: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_send_videos: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_send_video_notes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_send_voice_notes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_send_polls: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_send_other_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_add_web_page_previews: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_change_info: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_invite_users: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_pin_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_manage_topics: Option<bool>,
}

macro_rules! impl_chat_permissions_builders {
    ($($method:ident => $field:ident),+ $(,)?) => {
        $(
            pub fn $method(mut self, allowed: bool) -> Self {
                self.$field = Some(allowed);
                self
            }
        )+
    };
}

impl ChatPermissions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allow_all() -> Self {
        Self {
            can_send_messages: Some(true),
            can_send_audios: Some(true),
            can_send_documents: Some(true),
            can_send_photos: Some(true),
            can_send_videos: Some(true),
            can_send_video_notes: Some(true),
            can_send_voice_notes: Some(true),
            can_send_polls: Some(true),
            can_send_other_messages: Some(true),
            can_add_web_page_previews: Some(true),
            can_change_info: Some(true),
            can_invite_users: Some(true),
            can_pin_messages: Some(true),
            can_manage_topics: Some(true),
        }
    }

    pub fn deny_all() -> Self {
        Self {
            can_send_messages: Some(false),
            can_send_audios: Some(false),
            can_send_documents: Some(false),
            can_send_photos: Some(false),
            can_send_videos: Some(false),
            can_send_video_notes: Some(false),
            can_send_voice_notes: Some(false),
            can_send_polls: Some(false),
            can_send_other_messages: Some(false),
            can_add_web_page_previews: Some(false),
            can_change_info: Some(false),
            can_invite_users: Some(false),
            can_pin_messages: Some(false),
            can_manage_topics: Some(false),
        }
    }

    pub fn read_only() -> Self {
        Self::deny_all()
    }

    impl_chat_permissions_builders! {
        with_send_messages => can_send_messages,
        with_send_audios => can_send_audios,
        with_send_documents => can_send_documents,
        with_send_photos => can_send_photos,
        with_send_videos => can_send_videos,
        with_send_video_notes => can_send_video_notes,
        with_send_voice_notes => can_send_voice_notes,
        with_send_polls => can_send_polls,
        with_send_other_messages => can_send_other_messages,
        with_add_web_page_previews => can_add_web_page_previews,
        with_change_info => can_change_info,
        with_invite_users => can_invite_users,
        with_pin_messages => can_pin_messages,
        with_manage_topics => can_manage_topics,
    }
}

/// Telegram chat administrator rights object.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatAdministratorRights {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_manage_chat: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_delete_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_manage_video_chats: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_restrict_members: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_promote_members: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_change_info: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_invite_users: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_post_stories: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_edit_stories: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_delete_stories: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_post_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_edit_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_pin_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_manage_topics: Option<bool>,
}

impl ChatAdministratorRights {
    pub fn has_capability(&self, capability: ChatAdministratorCapability) -> bool {
        match capability {
            ChatAdministratorCapability::ManageChat => self.can_manage_chat.unwrap_or(false),
            ChatAdministratorCapability::DeleteMessages => {
                self.can_delete_messages.unwrap_or(false)
            }
            ChatAdministratorCapability::ManageVideoChats => {
                self.can_manage_video_chats.unwrap_or(false)
            }
            ChatAdministratorCapability::RestrictMembers => {
                self.can_restrict_members.unwrap_or(false)
            }
            ChatAdministratorCapability::PromoteMembers => {
                self.can_promote_members.unwrap_or(false)
            }
            ChatAdministratorCapability::ChangeInfo => self.can_change_info.unwrap_or(false),
            ChatAdministratorCapability::InviteUsers => self.can_invite_users.unwrap_or(false),
            ChatAdministratorCapability::PostStories => self.can_post_stories.unwrap_or(false),
            ChatAdministratorCapability::EditStories => self.can_edit_stories.unwrap_or(false),
            ChatAdministratorCapability::DeleteStories => self.can_delete_stories.unwrap_or(false),
            ChatAdministratorCapability::PostMessages => self.can_post_messages.unwrap_or(false),
            ChatAdministratorCapability::EditMessages => self.can_edit_messages.unwrap_or(false),
            ChatAdministratorCapability::PinMessages => self.can_pin_messages.unwrap_or(false),
            ChatAdministratorCapability::ManageTopics => self.can_manage_topics.unwrap_or(false),
        }
    }
}

fn chat_administrator_member_rights_len(rights: &ChatAdministratorRights) -> usize {
    usize::from(rights.can_manage_chat.is_some())
        + usize::from(rights.can_delete_messages.is_some())
        + usize::from(rights.can_manage_video_chats.is_some())
        + usize::from(rights.can_restrict_members.is_some())
        + usize::from(rights.can_promote_members.is_some())
        + usize::from(rights.can_change_info.is_some())
        + usize::from(rights.can_invite_users.is_some())
        + usize::from(rights.can_post_stories.is_some())
        + usize::from(rights.can_edit_stories.is_some())
        + usize::from(rights.can_delete_stories.is_some())
        + usize::from(rights.can_post_messages.is_some())
        + usize::from(rights.can_edit_messages.is_some())
        + usize::from(rights.can_pin_messages.is_some())
        + usize::from(rights.can_manage_topics.is_some())
}

fn serialize_chat_administrator_member_rights<M>(
    object: &mut M,
    rights: &ChatAdministratorRights,
) -> std::result::Result<(), M::Error>
where
    M: SerializeMap,
{
    if let Some(value) = rights.can_manage_chat {
        object.serialize_entry("can_manage_chat", &value)?;
    }
    if let Some(value) = rights.can_delete_messages {
        object.serialize_entry("can_delete_messages", &value)?;
    }
    if let Some(value) = rights.can_manage_video_chats {
        object.serialize_entry("can_manage_video_chats", &value)?;
    }
    if let Some(value) = rights.can_restrict_members {
        object.serialize_entry("can_restrict_members", &value)?;
    }
    if let Some(value) = rights.can_promote_members {
        object.serialize_entry("can_promote_members", &value)?;
    }
    if let Some(value) = rights.can_change_info {
        object.serialize_entry("can_change_info", &value)?;
    }
    if let Some(value) = rights.can_invite_users {
        object.serialize_entry("can_invite_users", &value)?;
    }
    if let Some(value) = rights.can_post_stories {
        object.serialize_entry("can_post_stories", &value)?;
    }
    if let Some(value) = rights.can_edit_stories {
        object.serialize_entry("can_edit_stories", &value)?;
    }
    if let Some(value) = rights.can_delete_stories {
        object.serialize_entry("can_delete_stories", &value)?;
    }
    if let Some(value) = rights.can_post_messages {
        object.serialize_entry("can_post_messages", &value)?;
    }
    if let Some(value) = rights.can_edit_messages {
        object.serialize_entry("can_edit_messages", &value)?;
    }
    if let Some(value) = rights.can_pin_messages {
        object.serialize_entry("can_pin_messages", &value)?;
    }
    if let Some(value) = rights.can_manage_topics {
        object.serialize_entry("can_manage_topics", &value)?;
    }

    Ok(())
}

fn chat_permissions_len(permissions: &ChatPermissions) -> usize {
    usize::from(permissions.can_send_messages.is_some())
        + usize::from(permissions.can_send_audios.is_some())
        + usize::from(permissions.can_send_documents.is_some())
        + usize::from(permissions.can_send_photos.is_some())
        + usize::from(permissions.can_send_videos.is_some())
        + usize::from(permissions.can_send_video_notes.is_some())
        + usize::from(permissions.can_send_voice_notes.is_some())
        + usize::from(permissions.can_send_polls.is_some())
        + usize::from(permissions.can_send_other_messages.is_some())
        + usize::from(permissions.can_add_web_page_previews.is_some())
        + usize::from(permissions.can_change_info.is_some())
        + usize::from(permissions.can_invite_users.is_some())
        + usize::from(permissions.can_pin_messages.is_some())
        + usize::from(permissions.can_manage_topics.is_some())
}

fn serialize_chat_permissions<M>(
    object: &mut M,
    permissions: &ChatPermissions,
) -> std::result::Result<(), M::Error>
where
    M: SerializeMap,
{
    if let Some(value) = permissions.can_send_messages {
        object.serialize_entry("can_send_messages", &value)?;
    }
    if let Some(value) = permissions.can_send_audios {
        object.serialize_entry("can_send_audios", &value)?;
    }
    if let Some(value) = permissions.can_send_documents {
        object.serialize_entry("can_send_documents", &value)?;
    }
    if let Some(value) = permissions.can_send_photos {
        object.serialize_entry("can_send_photos", &value)?;
    }
    if let Some(value) = permissions.can_send_videos {
        object.serialize_entry("can_send_videos", &value)?;
    }
    if let Some(value) = permissions.can_send_video_notes {
        object.serialize_entry("can_send_video_notes", &value)?;
    }
    if let Some(value) = permissions.can_send_voice_notes {
        object.serialize_entry("can_send_voice_notes", &value)?;
    }
    if let Some(value) = permissions.can_send_polls {
        object.serialize_entry("can_send_polls", &value)?;
    }
    if let Some(value) = permissions.can_send_other_messages {
        object.serialize_entry("can_send_other_messages", &value)?;
    }
    if let Some(value) = permissions.can_add_web_page_previews {
        object.serialize_entry("can_add_web_page_previews", &value)?;
    }
    if let Some(value) = permissions.can_change_info {
        object.serialize_entry("can_change_info", &value)?;
    }
    if let Some(value) = permissions.can_invite_users {
        object.serialize_entry("can_invite_users", &value)?;
    }
    if let Some(value) = permissions.can_pin_messages {
        object.serialize_entry("can_pin_messages", &value)?;
    }
    if let Some(value) = permissions.can_manage_topics {
        object.serialize_entry("can_manage_topics", &value)?;
    }

    Ok(())
}

/// Strongly typed chat member status.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ChatMemberStatus {
    Owner,
    Administrator,
    Member,
    Restricted,
    Left,
    Banned,
    Unknown(String),
}

impl ChatMemberStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Owner => "creator",
            Self::Administrator => "administrator",
            Self::Member => "member",
            Self::Restricted => "restricted",
            Self::Left => "left",
            Self::Banned => "kicked",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl From<&str> for ChatMemberStatus {
    fn from(value: &str) -> Self {
        match value {
            "creator" => Self::Owner,
            "administrator" => Self::Administrator,
            "member" => Self::Member,
            "restricted" => Self::Restricted,
            "left" => Self::Left,
            "kicked" => Self::Banned,
            _ => Self::Unknown(value.to_owned()),
        }
    }
}

impl<'de> Deserialize<'de> for ChatMemberStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(Self::from(value.as_str()))
    }
}

impl Serialize for ChatMemberStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Telegram chat owner payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatMemberOwner {
    pub user: User,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_title: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ChatMemberOwner {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["user", "is_anonymous", "custom_title"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len =
            usize::from(self.is_anonymous.is_some()) + usize::from(self.custom_title.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("user", &self.user)?;
        if let Some(is_anonymous) = self.is_anonymous {
            object.serialize_entry("is_anonymous", &is_anonymous)?;
        }
        if let Some(custom_title) = self.custom_title.as_ref() {
            object.serialize_entry("custom_title", custom_title)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram chat administrator payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatMemberAdministrator {
    pub user: User,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_be_edited: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_title: Option<String>,
    #[serde(flatten)]
    pub rights: ChatAdministratorRights,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ChatMemberAdministrator {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "user",
            "can_be_edited",
            "is_anonymous",
            "custom_title",
            "can_manage_chat",
            "can_delete_messages",
            "can_manage_video_chats",
            "can_restrict_members",
            "can_promote_members",
            "can_change_info",
            "can_invite_users",
            "can_post_stories",
            "can_edit_stories",
            "can_delete_stories",
            "can_post_messages",
            "can_edit_messages",
            "can_pin_messages",
            "can_manage_topics",
        ];
        let is_anonymous = self.is_anonymous.or(self.rights.is_anonymous);
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.can_be_edited.is_some())
            + usize::from(is_anonymous.is_some())
            + usize::from(self.custom_title.is_some())
            + chat_administrator_member_rights_len(&self.rights);
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("user", &self.user)?;
        if let Some(can_be_edited) = self.can_be_edited {
            object.serialize_entry("can_be_edited", &can_be_edited)?;
        }
        if let Some(is_anonymous) = is_anonymous {
            object.serialize_entry("is_anonymous", &is_anonymous)?;
        }
        if let Some(custom_title) = self.custom_title.as_ref() {
            object.serialize_entry("custom_title", custom_title)?;
        }
        serialize_chat_administrator_member_rights(&mut object, &self.rights)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram regular member payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatMemberRegular {
    pub user: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ChatMemberRegular {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["user"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("user", &self.user)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram restricted member payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatMemberRestricted {
    pub user: User,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_member: Option<bool>,
    #[serde(flatten)]
    pub permissions: ChatPermissions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ChatMemberRestricted {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "user",
            "is_member",
            "until_date",
            "can_send_messages",
            "can_send_audios",
            "can_send_documents",
            "can_send_photos",
            "can_send_videos",
            "can_send_video_notes",
            "can_send_voice_notes",
            "can_send_polls",
            "can_send_other_messages",
            "can_add_web_page_previews",
            "can_change_info",
            "can_invite_users",
            "can_pin_messages",
            "can_manage_topics",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.is_member.is_some())
            + usize::from(self.until_date.is_some())
            + chat_permissions_len(&self.permissions);
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("user", &self.user)?;
        if let Some(is_member) = self.is_member {
            object.serialize_entry("is_member", &is_member)?;
        }
        serialize_chat_permissions(&mut object, &self.permissions)?;
        if let Some(until_date) = self.until_date {
            object.serialize_entry("until_date", &until_date)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram left member payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatMemberLeft {
    pub user: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ChatMemberLeft {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["user"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 1))?;
        object.serialize_entry("user", &self.user)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram banned member payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct ChatMemberBanned {
    pub user: User,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for ChatMemberBanned {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["user", "until_date"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.until_date.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("user", &self.user)?;
        if let Some(until_date) = self.until_date {
            object.serialize_entry("until_date", &until_date)?;
        }
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram chat member object.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ChatMember {
    Owner(ChatMemberOwner),
    Administrator(ChatMemberAdministrator),
    Member(ChatMemberRegular),
    Restricted(ChatMemberRestricted),
    Left(ChatMemberLeft),
    Banned(ChatMemberBanned),
    Unknown(Value),
}

impl<'de> Deserialize<'de> for ChatMember {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match tagged_field(&value, "status") {
            Some("creator") => serde_json::from_value(strip_tag(value, "status"))
                .map(Self::Owner)
                .map_err(serde::de::Error::custom),
            Some("administrator") => serde_json::from_value(strip_tag(value, "status"))
                .map(Self::Administrator)
                .map_err(serde::de::Error::custom),
            Some("member") => serde_json::from_value(strip_tag(value, "status"))
                .map(Self::Member)
                .map_err(serde::de::Error::custom),
            Some("restricted") => serde_json::from_value(strip_tag(value, "status"))
                .map(Self::Restricted)
                .map_err(serde::de::Error::custom),
            Some("left") => serde_json::from_value(strip_tag(value, "status"))
                .map(Self::Left)
                .map_err(serde::de::Error::custom),
            Some("kicked") => serde_json::from_value(strip_tag(value, "status"))
                .map(Self::Banned)
                .map_err(serde::de::Error::custom),
            Some(_) | None => Ok(Self::Unknown(value)),
        }
    }
}

impl Serialize for ChatMember {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Owner(value) => serialize_tagged_field(serializer, "status", "creator", value),
            Self::Administrator(value) => {
                serialize_tagged_field(serializer, "status", "administrator", value)
            }
            Self::Member(value) => serialize_tagged_field(serializer, "status", "member", value),
            Self::Restricted(value) => {
                serialize_tagged_field(serializer, "status", "restricted", value)
            }
            Self::Left(value) => serialize_tagged_field(serializer, "status", "left", value),
            Self::Banned(value) => serialize_tagged_field(serializer, "status", "kicked", value),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

/// Administrative capabilities exposed by `getChatMember`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ChatAdministratorCapability {
    ManageChat,
    DeleteMessages,
    ManageVideoChats,
    RestrictMembers,
    PromoteMembers,
    ChangeInfo,
    InviteUsers,
    PostStories,
    EditStories,
    DeleteStories,
    PostMessages,
    EditMessages,
    PinMessages,
    ManageTopics,
}

impl ChatAdministratorCapability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ManageChat => "manage_chat",
            Self::DeleteMessages => "delete_messages",
            Self::ManageVideoChats => "manage_video_chats",
            Self::RestrictMembers => "restrict_members",
            Self::PromoteMembers => "promote_members",
            Self::ChangeInfo => "change_info",
            Self::InviteUsers => "invite_users",
            Self::PostStories => "post_stories",
            Self::EditStories => "edit_stories",
            Self::DeleteStories => "delete_stories",
            Self::PostMessages => "post_messages",
            Self::EditMessages => "edit_messages",
            Self::PinMessages => "pin_messages",
            Self::ManageTopics => "manage_topics",
        }
    }
}

impl ChatMember {
    pub fn status_name(&self) -> Option<&str> {
        match self {
            Self::Owner(_) => Some("creator"),
            Self::Administrator(_) => Some("administrator"),
            Self::Member(_) => Some("member"),
            Self::Restricted(_) => Some("restricted"),
            Self::Left(_) => Some("left"),
            Self::Banned(_) => Some("kicked"),
            Self::Unknown(value) => tagged_field(value, "status"),
        }
    }

    pub fn status(&self) -> Option<ChatMemberStatus> {
        self.status_name().map(ChatMemberStatus::from)
    }

    pub fn user(&self) -> Option<&User> {
        match self {
            Self::Owner(member) => Some(&member.user),
            Self::Administrator(member) => Some(&member.user),
            Self::Member(member) => Some(&member.user),
            Self::Restricted(member) => Some(&member.user),
            Self::Left(member) => Some(&member.user),
            Self::Banned(member) => Some(&member.user),
            Self::Unknown(_) => None,
        }
    }

    pub fn as_owner(&self) -> Option<&ChatMemberOwner> {
        match self {
            Self::Owner(member) => Some(member),
            Self::Administrator(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_owner(self) -> Option<ChatMemberOwner> {
        match self {
            Self::Owner(member) => Some(member),
            Self::Administrator(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_administrator(&self) -> Option<&ChatMemberAdministrator> {
        match self {
            Self::Administrator(member) => Some(member),
            Self::Owner(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_administrator(self) -> Option<ChatMemberAdministrator> {
        match self {
            Self::Administrator(member) => Some(member),
            Self::Owner(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_member(&self) -> Option<&ChatMemberRegular> {
        match self {
            Self::Member(member) => Some(member),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_member(self) -> Option<ChatMemberRegular> {
        match self {
            Self::Member(member) => Some(member),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_restricted(&self) -> Option<&ChatMemberRestricted> {
        match self {
            Self::Restricted(member) => Some(member),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Member(_)
            | Self::Left(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_restricted(self) -> Option<ChatMemberRestricted> {
        match self {
            Self::Restricted(member) => Some(member),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Member(_)
            | Self::Left(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_left(&self) -> Option<&ChatMemberLeft> {
        match self {
            Self::Left(member) => Some(member),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_left(self) -> Option<ChatMemberLeft> {
        match self {
            Self::Left(member) => Some(member),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_banned(&self) -> Option<&ChatMemberBanned> {
        match self {
            Self::Banned(member) => Some(member),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_banned(self) -> Option<ChatMemberBanned> {
        match self {
            Self::Banned(member) => Some(member),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_unknown_value(&self) -> Option<&Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_) => None,
        }
    }

    pub fn into_unknown_value(self) -> Option<Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_) => None,
        }
    }

    pub fn custom_title(&self) -> Option<&str> {
        match self {
            Self::Owner(member) => member.custom_title.as_deref(),
            Self::Administrator(member) => member.custom_title.as_deref(),
            Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn administrator_rights(&self) -> Option<&ChatAdministratorRights> {
        self.as_administrator().map(|member| &member.rights)
    }

    pub fn permissions(&self) -> Option<&ChatPermissions> {
        self.as_restricted().map(|member| &member.permissions)
    }

    pub fn until_date(&self) -> Option<i64> {
        match self {
            Self::Restricted(member) => member.until_date,
            Self::Banned(member) => member.until_date,
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Member(_)
            | Self::Left(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn extra(&self) -> Option<&BTreeMap<String, Value>> {
        match self {
            Self::Owner(member) => Some(&member.extra),
            Self::Administrator(member) => Some(&member.extra),
            Self::Member(member) => Some(&member.extra),
            Self::Restricted(member) => Some(&member.extra),
            Self::Left(member) => Some(&member.extra),
            Self::Banned(member) => Some(&member.extra),
            Self::Unknown(_) => None,
        }
    }

    pub fn is_owner(&self) -> bool {
        matches!(self, Self::Owner(_))
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Self::Owner(_) | Self::Administrator(_))
    }

    pub fn has_capability(&self, capability: ChatAdministratorCapability) -> bool {
        match self {
            Self::Owner(_) => true,
            Self::Administrator(member) => member.rights.has_capability(capability),
            Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_)
            | Self::Unknown(_) => false,
        }
    }
}

/// Telegram chat invite link object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatInviteLink {
    pub invite_link: String,
    pub creator: User,
    pub creates_join_request: bool,
    pub is_primary: bool,
    pub is_revoked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expire_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_join_request_count: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GetChatRequest {
    pub chat_id: ChatId,
}

#[derive(Clone, Debug, Serialize)]
pub struct GetChatAdministratorsRequest {
    pub chat_id: ChatId,
}

#[derive(Clone, Debug, Serialize)]
pub struct GetChatMemberCountRequest {
    pub chat_id: ChatId,
}

impl GetChatMemberCountRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct GetChatMemberRequest {
    pub chat_id: ChatId,
    pub user_id: UserId,
}

#[derive(Clone, Debug, Serialize)]
pub struct LeaveChatRequest {
    pub chat_id: ChatId,
}

#[derive(Clone, Debug, Serialize)]
pub struct BanChatMemberRequest {
    pub chat_id: ChatId,
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revoke_messages: Option<bool>,
}

impl BanChatMemberRequest {
    pub fn new(chat_id: impl Into<ChatId>, user_id: impl Into<UserId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            user_id: user_id.into(),
            until_date: None,
            revoke_messages: None,
        }
    }

    pub fn until_date(mut self, until_date: i64) -> Self {
        self.until_date = Some(until_date);
        self
    }

    pub fn revoke_messages(mut self, revoke_messages: bool) -> Self {
        self.revoke_messages = Some(revoke_messages);
        self
    }

    pub fn with_until_date(mut self, until_date: Option<i64>) -> Self {
        self.until_date = until_date;
        self
    }

    pub fn with_revoke_messages(mut self, revoke_messages: Option<bool>) -> Self {
        self.revoke_messages = revoke_messages;
        self
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct UnbanChatMemberRequest {
    pub chat_id: ChatId,
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub only_if_banned: Option<bool>,
}

impl UnbanChatMemberRequest {
    pub fn new(chat_id: impl Into<ChatId>, user_id: impl Into<UserId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            user_id: user_id.into(),
            only_if_banned: None,
        }
    }

    pub fn only_if_banned(mut self, only_if_banned: bool) -> Self {
        self.only_if_banned = Some(only_if_banned);
        self
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RestrictChatMemberRequest {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub permissions: ChatPermissions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_independent_chat_permissions: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_date: Option<i64>,
}

impl RestrictChatMemberRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        permissions: ChatPermissions,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            user_id: user_id.into(),
            permissions,
            use_independent_chat_permissions: None,
            until_date: None,
        }
    }

    pub fn use_independent_chat_permissions(
        mut self,
        use_independent_chat_permissions: bool,
    ) -> Self {
        self.use_independent_chat_permissions = Some(use_independent_chat_permissions);
        self
    }

    pub fn until_date(mut self, until_date: i64) -> Self {
        self.until_date = Some(until_date);
        self
    }

    pub fn with_use_independent_chat_permissions(
        mut self,
        use_independent_chat_permissions: Option<bool>,
    ) -> Self {
        self.use_independent_chat_permissions = use_independent_chat_permissions;
        self
    }

    pub fn with_until_date(mut self, until_date: Option<i64>) -> Self {
        self.until_date = until_date;
        self
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PromoteChatMemberRequest {
    pub chat_id: ChatId,
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_manage_chat: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_delete_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_manage_video_chats: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_restrict_members: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_promote_members: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_change_info: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_invite_users: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_post_stories: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_edit_stories: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_delete_stories: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_post_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_edit_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_pin_messages: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_manage_topics: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SetChatAdministratorCustomTitleRequest {
    pub chat_id: ChatId,
    pub user_id: UserId,
    pub custom_title: String,
}

impl SetChatAdministratorCustomTitleRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        self.user_id.validate()?;
        validate_text_limit(
            "custom_title",
            &self.custom_title,
            TextPresence::AllowEmpty,
            TextLayout::SingleLine,
            MAX_CUSTOM_TITLE_CHARS,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BanChatSenderChatRequest {
    pub chat_id: ChatId,
    pub sender_chat_id: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnbanChatSenderChatRequest {
    pub chat_id: ChatId,
    pub sender_chat_id: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct SetChatPermissionsRequest {
    pub chat_id: ChatId,
    pub permissions: ChatPermissions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_independent_chat_permissions: Option<bool>,
}

impl SetChatPermissionsRequest {
    pub fn new(chat_id: impl Into<ChatId>, permissions: ChatPermissions) -> Self {
        Self {
            chat_id: chat_id.into(),
            permissions,
            use_independent_chat_permissions: None,
        }
    }

    pub fn use_independent_chat_permissions(
        mut self,
        use_independent_chat_permissions: bool,
    ) -> Self {
        self.use_independent_chat_permissions = Some(use_independent_chat_permissions);
        self
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ExportChatInviteLinkRequest {
    pub chat_id: ChatId,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateChatInviteLinkRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expire_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creates_join_request: Option<bool>,
}

impl CreateChatInviteLinkRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        validate_optional_unix_timestamp("expire_date", self.expire_date)?;
        validate_invite_link_options(
            self.name.as_deref(),
            self.member_limit,
            self.creates_join_request,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct EditChatInviteLinkRequest {
    pub chat_id: ChatId,
    pub invite_link: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expire_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creates_join_request: Option<bool>,
}

impl EditChatInviteLinkRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        validate_required_text("invite_link", &self.invite_link)?;
        validate_optional_unix_timestamp("expire_date", self.expire_date)?;
        validate_invite_link_options(
            self.name.as_deref(),
            self.member_limit,
            self.creates_join_request,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RevokeChatInviteLinkRequest {
    pub chat_id: ChatId,
    pub invite_link: String,
}

impl RevokeChatInviteLinkRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        validate_required_text("invite_link", &self.invite_link)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SetChatTitleRequest {
    pub chat_id: ChatId,
    pub title: String,
}

impl SetChatTitleRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        validate_text_limit(
            "title",
            &self.title,
            TextPresence::Required,
            TextLayout::SingleLine,
            MAX_CHAT_TITLE_CHARS,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SetChatDescriptionRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl SetChatDescriptionRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        let Some(description) = self.description.as_deref() else {
            return Ok(());
        };

        validate_text_limit(
            "description",
            description,
            TextPresence::AllowEmpty,
            TextLayout::MultiLine,
            MAX_CHAT_DESCRIPTION_CHARS,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PinChatMessageRequest {
    pub chat_id: ChatId,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnpinChatMessageRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnpinAllChatMessagesRequest {
    pub chat_id: ChatId,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeleteChatPhotoRequest {
    pub chat_id: ChatId,
}

#[derive(Clone, Debug, Serialize)]
pub struct SetChatPhotoRequest {
    pub chat_id: ChatId,
}

impl SetChatPhotoRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SetChatStickerSetRequest {
    pub chat_id: ChatId,
    pub sticker_set_name: String,
}

impl SetChatStickerSetRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        validate_text_limit(
            "sticker_set_name",
            &self.sticker_set_name,
            TextPresence::Required,
            TextLayout::SingleLine,
            MAX_STICKER_SET_NAME_CHARS,
        )
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DeleteChatStickerSetRequest {
    pub chat_id: ChatId,
}

macro_rules! impl_chat_id_validate {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn validate(&self) -> Result<()> {
                    self.chat_id.validate()
                }
            }
        )*
    };
}

macro_rules! impl_chat_and_user_validate {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn validate(&self) -> Result<()> {
                    self.chat_id.validate()?;
                    self.user_id.validate()
                }
            }
        )*
    };
}

macro_rules! impl_chat_and_sender_chat_validate {
    ($($ty:ty),* $(,)?) => {
        $(
            impl $ty {
                pub fn validate(&self) -> Result<()> {
                    self.chat_id.validate()?;
                    ChatId::Id(self.sender_chat_id).validate()
                }
            }
        )*
    };
}

impl_chat_id_validate!(
    GetChatRequest,
    GetChatAdministratorsRequest,
    GetChatMemberCountRequest,
    LeaveChatRequest,
    SetChatPermissionsRequest,
    ExportChatInviteLinkRequest,
    UnpinAllChatMessagesRequest,
    DeleteChatPhotoRequest,
    SetChatPhotoRequest,
    DeleteChatStickerSetRequest,
);

impl_chat_and_user_validate!(
    GetChatMemberRequest,
    UnbanChatMemberRequest,
    PromoteChatMemberRequest,
);

impl_chat_and_sender_chat_validate!(BanChatSenderChatRequest, UnbanChatSenderChatRequest,);

impl BanChatMemberRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        self.user_id.validate()?;
        validate_optional_unix_timestamp("until_date", self.until_date)
    }
}

impl RestrictChatMemberRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        self.user_id.validate()?;
        validate_optional_unix_timestamp("until_date", self.until_date)
    }
}

impl PinChatMessageRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        self.message_id.validate()
    }
}

impl UnpinChatMessageRequest {
    pub fn validate(&self) -> Result<()> {
        self.chat_id.validate()?;
        if let Some(message_id) = self.message_id {
            message_id.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum TextPresence {
    Required,
    AllowEmpty,
}

#[derive(Clone, Copy)]
enum TextLayout {
    SingleLine,
    MultiLine,
}

fn validate_invite_link_options(
    name: Option<&str>,
    member_limit: Option<u32>,
    creates_join_request: Option<bool>,
) -> Result<()> {
    if let Some(name) = name {
        validate_text_limit(
            "name",
            name,
            TextPresence::AllowEmpty,
            TextLayout::SingleLine,
            MAX_INVITE_LINK_NAME_CHARS,
        )?;
    }

    if let Some(member_limit) = member_limit
        && !(1..=MAX_INVITE_LINK_MEMBER_LIMIT).contains(&member_limit)
    {
        return Err(Error::InvalidRequest {
            reason: format!("member_limit must be 1-{MAX_INVITE_LINK_MEMBER_LIMIT} users"),
        });
    }

    if creates_join_request == Some(true) && member_limit.is_some() {
        return Err(Error::InvalidRequest {
            reason: "creates_join_request cannot be combined with member_limit".to_owned(),
        });
    }

    Ok(())
}

fn validate_optional_unix_timestamp(field: &str, value: Option<i64>) -> Result<()> {
    if value.is_some_and(|value| value < 0) {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must not be negative"),
        });
    }

    Ok(())
}

fn validate_required_text(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must not be empty"),
        });
    }
    if value.chars().any(char::is_control) {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must not contain control characters"),
        });
    }

    Ok(())
}

fn validate_text_limit(
    field: &str,
    value: &str,
    presence: TextPresence,
    layout: TextLayout,
    max_chars: usize,
) -> Result<()> {
    if matches!(presence, TextPresence::Required) && value.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must not be empty"),
        });
    }

    if value.chars().count() > max_chars {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must be at most {max_chars} characters"),
        });
    }

    if value
        .chars()
        .any(|character| is_disallowed_control(character, layout))
    {
        return Err(Error::InvalidRequest {
            reason: format!("{field} must not contain control characters"),
        });
    }

    Ok(())
}

fn is_disallowed_control(character: char, layout: TextLayout) -> bool {
    match layout {
        TextLayout::SingleLine => character.is_control(),
        TextLayout::MultiLine => character.is_control() && !matches!(character, '\n' | '\r' | '\t'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn chat_permissions_presets_and_builders_work() {
        let denied = ChatPermissions::deny_all();
        assert_eq!(denied.can_send_messages, Some(false));
        assert_eq!(denied.can_manage_topics, Some(false));

        let allowed = ChatPermissions::allow_all();
        assert_eq!(allowed.can_send_messages, Some(true));
        assert_eq!(allowed.can_invite_users, Some(true));

        let custom = ChatPermissions::read_only()
            .with_send_messages(true)
            .with_add_web_page_previews(true);
        assert_eq!(custom.can_send_messages, Some(true));
        assert_eq!(custom.can_add_web_page_previews, Some(true));
        assert_eq!(custom.can_send_photos, Some(false));
    }

    #[test]
    fn validates_chat_profile_requests() {
        let title = SetChatTitleRequest {
            chat_id: ChatId::from(1),
            title: "group".to_owned(),
        };
        assert!(title.validate().is_ok());

        let empty_title = SetChatTitleRequest {
            chat_id: ChatId::from(1),
            title: "   ".to_owned(),
        };
        assert!(matches!(
            empty_title.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let long_description = SetChatDescriptionRequest {
            chat_id: ChatId::from(1),
            description: Some("a".repeat(MAX_CHAT_DESCRIPTION_CHARS + 1)),
        };
        assert!(matches!(
            long_description.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let multiline_description = SetChatDescriptionRequest {
            chat_id: ChatId::from(1),
            description: Some("hello\nworld".to_owned()),
        };
        assert!(multiline_description.validate().is_ok());

        let multiline_title = SetChatTitleRequest {
            chat_id: ChatId::from(1),
            title: "hello\nworld".to_owned(),
        };
        assert!(matches!(
            multiline_title.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let custom_title = SetChatAdministratorCustomTitleRequest {
            chat_id: ChatId::from(1),
            user_id: UserId::from(2),
            custom_title: "a".repeat(MAX_CUSTOM_TITLE_CHARS + 1),
        };
        assert!(matches!(
            custom_title.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let sticker_set = SetChatStickerSetRequest {
            chat_id: ChatId::from(1),
            sticker_set_name: "bad\nname".to_owned(),
        };
        assert!(matches!(
            sticker_set.validate(),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[test]
    fn validates_chat_id_on_chat_requests() {
        let get_chat = GetChatRequest {
            chat_id: ChatId::from(0),
        };
        assert!(matches!(
            get_chat.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let ban = BanChatMemberRequest::new("channel", UserId::from(2));
        assert!(matches!(ban.validate(), Err(Error::InvalidRequest { .. })));

        let invalid_user = BanChatMemberRequest::new(1, UserId::from(0));
        assert!(matches!(
            invalid_user.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_ban_until_date = BanChatMemberRequest::new(1, UserId::from(2)).until_date(-1);
        assert!(matches!(
            invalid_ban_until_date.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_restrict_until_date =
            RestrictChatMemberRequest::new(1, UserId::from(2), ChatPermissions::deny_all())
                .until_date(-1);
        assert!(matches!(
            invalid_restrict_until_date.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_sender_chat = BanChatSenderChatRequest {
            chat_id: ChatId::from(1),
            sender_chat_id: 0,
        };
        assert!(matches!(
            invalid_sender_chat.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invite = CreateChatInviteLinkRequest {
            chat_id: ChatId::from("channel"),
            name: None,
            expire_date: None,
            member_limit: None,
            creates_join_request: None,
        };
        assert!(matches!(
            invite.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let pin = PinChatMessageRequest {
            chat_id: ChatId::from("@channel"),
            message_id: MessageId::from(1),
            disable_notification: None,
        };
        assert!(pin.validate().is_ok());

        let invalid_pin = PinChatMessageRequest {
            chat_id: ChatId::from("@channel"),
            message_id: MessageId::from(0),
            disable_notification: None,
        };
        assert!(matches!(
            invalid_pin.validate(),
            Err(Error::InvalidRequest { .. })
        ));
    }

    #[test]
    fn validates_chat_invite_link_requests() {
        let valid = CreateChatInviteLinkRequest {
            chat_id: ChatId::from(1),
            name: Some("mods".to_owned()),
            expire_date: None,
            member_limit: Some(10),
            creates_join_request: None,
        };
        assert!(valid.validate().is_ok());

        let invalid_limit = CreateChatInviteLinkRequest {
            member_limit: Some(0),
            ..valid.clone()
        };
        assert!(matches!(
            invalid_limit.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let invalid_expire_date = CreateChatInviteLinkRequest {
            expire_date: Some(-1),
            ..valid.clone()
        };
        assert!(matches!(
            invalid_expire_date.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let join_request_with_limit = CreateChatInviteLinkRequest {
            member_limit: Some(10),
            creates_join_request: Some(true),
            ..valid
        };
        assert!(matches!(
            join_request_with_limit.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let edit = EditChatInviteLinkRequest {
            chat_id: ChatId::from(1),
            invite_link: "".to_owned(),
            name: Some("a".repeat(MAX_INVITE_LINK_NAME_CHARS + 1)),
            expire_date: None,
            member_limit: None,
            creates_join_request: None,
        };
        assert!(matches!(edit.validate(), Err(Error::InvalidRequest { .. })));

        let revoke = RevokeChatInviteLinkRequest {
            chat_id: ChatId::from(1),
            invite_link: "https://t.me/+abc".to_owned(),
        };
        assert!(revoke.validate().is_ok());
    }

    #[test]
    fn chat_member_capabilities_are_fully_typed()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let member: ChatMember = serde_json::from_value(json!({
            "status": "administrator",
            "user": {"id": 1, "is_bot": false, "first_name": "mod"},
            "can_manage_chat": true,
            "can_delete_messages": true,
            "can_manage_video_chats": true,
            "can_restrict_members": true,
            "can_promote_members": false,
            "can_change_info": true,
            "can_invite_users": true,
            "can_post_stories": true,
            "can_edit_stories": false,
            "can_delete_stories": true,
            "can_post_messages": false,
            "can_edit_messages": false,
            "can_pin_messages": true,
            "can_manage_topics": true
        }))?;

        assert_eq!(member.status_name(), Some("administrator"));
        assert_eq!(member.status(), Some(ChatMemberStatus::Administrator));
        assert_eq!(member.user().map(|user| user.id.0), Some(1));
        assert!(member.as_administrator().is_some());
        assert_eq!(
            member
                .clone()
                .into_administrator()
                .map(|administrator| administrator.user.first_name),
            Some("mod".to_owned())
        );
        assert!(member.has_capability(ChatAdministratorCapability::ManageChat));
        assert!(member.has_capability(ChatAdministratorCapability::DeleteMessages));
        assert!(member.has_capability(ChatAdministratorCapability::ManageVideoChats));
        assert!(member.has_capability(ChatAdministratorCapability::RestrictMembers));
        assert!(!member.has_capability(ChatAdministratorCapability::PromoteMembers));
        assert!(member.has_capability(ChatAdministratorCapability::ChangeInfo));
        assert!(member.has_capability(ChatAdministratorCapability::InviteUsers));
        assert!(member.has_capability(ChatAdministratorCapability::PostStories));
        assert!(!member.has_capability(ChatAdministratorCapability::EditStories));
        assert!(member.has_capability(ChatAdministratorCapability::DeleteStories));
        assert!(!member.has_capability(ChatAdministratorCapability::PostMessages));
        assert!(!member.has_capability(ChatAdministratorCapability::EditMessages));
        assert!(member.has_capability(ChatAdministratorCapability::PinMessages));
        assert!(member.has_capability(ChatAdministratorCapability::ManageTopics));

        Ok(())
    }

    #[test]
    fn chat_member_extra_cannot_override_reserved_fields()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut administrator = ChatMemberAdministrator {
            user: User {
                id: UserId(1),
                is_bot: false,
                first_name: "mod".to_owned(),
                last_name: None,
                username: None,
                language_code: None,
                extra: BTreeMap::new(),
            },
            can_be_edited: Some(true),
            is_anonymous: Some(true),
            custom_title: Some("Owner".to_owned()),
            rights: ChatAdministratorRights {
                can_manage_chat: Some(true),
                is_anonymous: Some(false),
                ..ChatAdministratorRights::default()
            },
            extra: BTreeMap::new(),
        };
        administrator
            .extra
            .insert("status".to_owned(), json!("member"));
        administrator.extra.insert(
            "user".to_owned(),
            json!({"id": 9, "is_bot": true, "first_name": "spoofed"}),
        );
        administrator
            .extra
            .insert("is_anonymous".to_owned(), json!(false));
        administrator
            .extra
            .insert("can_manage_chat".to_owned(), json!(false));
        administrator
            .extra
            .insert("future_field".to_owned(), json!("kept"));

        let admin_value = serde_json::to_value(ChatMember::Administrator(administrator))?;
        assert_eq!(admin_value["status"], "administrator");
        assert_eq!(admin_value["user"]["id"], 1);
        assert_eq!(admin_value["user"]["first_name"], "mod");
        assert_eq!(admin_value["is_anonymous"], true);
        assert_eq!(admin_value["can_be_edited"], true);
        assert_eq!(admin_value["can_manage_chat"], true);
        assert_eq!(admin_value["future_field"], "kept");

        let mut restricted = ChatMemberRestricted {
            user: User {
                id: UserId(2),
                is_bot: false,
                first_name: "member".to_owned(),
                last_name: None,
                username: None,
                language_code: None,
                extra: BTreeMap::new(),
            },
            is_member: Some(true),
            permissions: ChatPermissions::new().with_send_messages(true),
            until_date: Some(1_700_000_000),
            extra: BTreeMap::new(),
        };
        restricted
            .extra
            .insert("status".to_owned(), json!("administrator"));
        restricted.extra.insert(
            "user".to_owned(),
            json!({"id": 9, "is_bot": true, "first_name": "spoofed"}),
        );
        restricted
            .extra
            .insert("can_send_messages".to_owned(), json!(false));
        restricted.extra.insert("until_date".to_owned(), json!(1));
        restricted
            .extra
            .insert("future_field".to_owned(), json!("kept"));

        let restricted_value = serde_json::to_value(ChatMember::Restricted(restricted))?;
        assert_eq!(restricted_value["status"], "restricted");
        assert_eq!(restricted_value["user"]["id"], 2);
        assert_eq!(restricted_value["user"]["first_name"], "member");
        assert_eq!(restricted_value["can_send_messages"], true);
        assert_eq!(restricted_value["until_date"], 1_700_000_000);
        assert_eq!(restricted_value["future_field"], "kept");

        Ok(())
    }

    #[test]
    fn unknown_chat_member_status_is_preserved()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let input = json!({
            "status": "future_status",
            "user": {"id": 2, "is_bot": false, "first_name": "future"},
            "payload": {"kept": true}
        });
        let member: ChatMember = serde_json::from_value(input.clone())?;

        assert_eq!(member.status_name(), Some("future_status"));
        assert_eq!(
            member.status(),
            Some(ChatMemberStatus::Unknown("future_status".to_owned()))
        );
        assert!(member.user().is_none());
        assert!(!member.is_admin());
        assert!(!member.has_capability(ChatAdministratorCapability::ManageChat));
        assert_eq!(member.as_unknown_value(), Some(&input));
        assert_eq!(member.clone().into_unknown_value(), Some(input.clone()));
        assert_eq!(serde_json::to_value(member)?, input);

        Ok(())
    }
}
