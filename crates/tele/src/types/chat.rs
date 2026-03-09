use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::bot::User;
use crate::types::common::{ChatId, MessageId, UserId};

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

/// Strongly typed chat member status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub enum ChatMemberStatus {
    #[serde(rename = "creator")]
    Owner,
    #[serde(rename = "administrator")]
    Administrator,
    #[serde(rename = "member")]
    Member,
    #[serde(rename = "restricted")]
    Restricted,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "kicked")]
    Banned,
}

/// Telegram chat owner payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

/// Telegram chat administrator payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

/// Telegram regular member payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatMemberRegular {
    pub user: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram restricted member payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

/// Telegram left member payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatMemberLeft {
    pub user: User,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram banned member payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatMemberBanned {
    pub user: User,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram chat member object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "status")]
#[non_exhaustive]
pub enum ChatMember {
    #[serde(rename = "creator")]
    Owner(ChatMemberOwner),
    #[serde(rename = "administrator")]
    Administrator(ChatMemberAdministrator),
    #[serde(rename = "member")]
    Member(ChatMemberRegular),
    #[serde(rename = "restricted")]
    Restricted(ChatMemberRestricted),
    #[serde(rename = "left")]
    Left(ChatMemberLeft),
    #[serde(rename = "kicked")]
    Banned(ChatMemberBanned),
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
    pub fn status(&self) -> ChatMemberStatus {
        match self {
            Self::Owner(_) => ChatMemberStatus::Owner,
            Self::Administrator(_) => ChatMemberStatus::Administrator,
            Self::Member(_) => ChatMemberStatus::Member,
            Self::Restricted(_) => ChatMemberStatus::Restricted,
            Self::Left(_) => ChatMemberStatus::Left,
            Self::Banned(_) => ChatMemberStatus::Banned,
        }
    }

    pub fn user(&self) -> &User {
        match self {
            Self::Owner(member) => &member.user,
            Self::Administrator(member) => &member.user,
            Self::Member(member) => &member.user,
            Self::Restricted(member) => &member.user,
            Self::Left(member) => &member.user,
            Self::Banned(member) => &member.user,
        }
    }

    pub fn custom_title(&self) -> Option<&str> {
        match self {
            Self::Owner(member) => member.custom_title.as_deref(),
            Self::Administrator(member) => member.custom_title.as_deref(),
            Self::Member(_) | Self::Restricted(_) | Self::Left(_) | Self::Banned(_) => None,
        }
    }

    pub fn administrator_rights(&self) -> Option<&ChatAdministratorRights> {
        match self {
            Self::Administrator(member) => Some(&member.rights),
            Self::Owner(_)
            | Self::Member(_)
            | Self::Restricted(_)
            | Self::Left(_)
            | Self::Banned(_) => None,
        }
    }

    pub fn permissions(&self) -> Option<&ChatPermissions> {
        match self {
            Self::Restricted(member) => Some(&member.permissions),
            Self::Owner(_)
            | Self::Administrator(_)
            | Self::Member(_)
            | Self::Left(_)
            | Self::Banned(_) => None,
        }
    }

    pub fn until_date(&self) -> Option<i64> {
        match self {
            Self::Restricted(member) => member.until_date,
            Self::Banned(member) => member.until_date,
            Self::Owner(_) | Self::Administrator(_) | Self::Member(_) | Self::Left(_) => None,
        }
    }

    pub fn extra(&self) -> &BTreeMap<String, Value> {
        match self {
            Self::Owner(member) => &member.extra,
            Self::Administrator(member) => &member.extra,
            Self::Member(member) => &member.extra,
            Self::Restricted(member) => &member.extra,
            Self::Left(member) => &member.extra,
            Self::Banned(member) => &member.extra,
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
            Self::Member(_) | Self::Restricted(_) | Self::Left(_) | Self::Banned(_) => false,
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

#[derive(Clone, Debug, Serialize)]
pub struct RevokeChatInviteLinkRequest {
    pub chat_id: ChatId,
    pub invite_link: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SetChatTitleRequest {
    pub chat_id: ChatId,
    pub title: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SetChatDescriptionRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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
pub struct SetChatStickerSetRequest {
    pub chat_id: ChatId,
    pub sticker_set_name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DeleteChatStickerSetRequest {
    pub chat_id: ChatId,
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

        assert_eq!(member.status(), ChatMemberStatus::Administrator);
        assert_eq!(member.user().id.0, 1);
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
}
