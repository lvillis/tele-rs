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

/// Telegram chat member object.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ChatMember {
    pub status: String,
    pub user: User,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_date: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_anonymous: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_manage_chat: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
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

#[derive(Clone, Debug, Serialize)]
pub struct UnbanChatMemberRequest {
    pub chat_id: ChatId,
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub only_if_banned: Option<bool>,
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
