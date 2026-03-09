use super::support::{invalid_request, update_message};
#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;
use crate::Result;
use crate::types::advanced::{AdvancedApproveChatJoinRequest, AdvancedDeclineChatJoinRequest};
use crate::types::chat::{BanChatMemberRequest, ChatPermissions, RestrictChatMemberRequest};
use crate::types::common::{ChatId, MessageId, UserId};
use crate::types::message::{DeleteMessageRequest, Message};
use crate::types::update::Update;

fn author_user_id(message: &Message, method: &str) -> Result<UserId> {
    message.from_user().map(|user| user.id).ok_or_else(|| {
        invalid_request(format!(
            "message does not contain a user sender for {method}"
        ))
    })
}

fn join_request_ids(update: &Update, method: &str) -> Result<(i64, UserId)> {
    let Some(request) = update.chat_join_request() else {
        return Err(invalid_request(format!(
            "update does not contain chat join request for {method}",
        )));
    };
    Ok((request.chat_id(), request.user_id().into()))
}

/// Optional fields for high-level `banChatMember` helpers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct BanMemberOptions {
    pub until_date: Option<i64>,
    pub revoke_messages: Option<bool>,
}

impl BanMemberOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn until_date(mut self, until_date: i64) -> Self {
        self.until_date = Some(until_date);
        self
    }

    pub fn revoke_messages(mut self, revoke_messages: bool) -> Self {
        self.revoke_messages = Some(revoke_messages);
        self
    }
}

/// Optional fields for high-level `restrictChatMember` helpers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct RestrictMemberOptions {
    pub use_independent_chat_permissions: Option<bool>,
    pub until_date: Option<i64>,
}

impl RestrictMemberOptions {
    pub fn new() -> Self {
        Self::default()
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
}

/// App-facing moderation/admin facade for runtime governance actions.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct ModerationApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl ModerationApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn approve_join_request(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<bool> {
        let request = AdvancedApproveChatJoinRequest::new(chat_id, user_id.into());
        self.client
            .advanced()
            .approve_chat_join_request_typed(&request)
            .await
    }

    pub async fn approve_join_request_from_update(&self, update: &Update) -> Result<bool> {
        let (chat_id, user_id) = join_request_ids(update, "approveChatJoinRequest")?;
        self.approve_join_request(chat_id, user_id).await
    }

    pub async fn decline_join_request(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<bool> {
        let request = AdvancedDeclineChatJoinRequest::new(chat_id, user_id.into());
        self.client
            .advanced()
            .decline_chat_join_request_typed(&request)
            .await
    }

    pub async fn decline_join_request_from_update(&self, update: &Update) -> Result<bool> {
        let (chat_id, user_id) = join_request_ids(update, "declineChatJoinRequest")?;
        self.decline_join_request(chat_id, user_id).await
    }

    pub async fn ban_member(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<bool> {
        self.ban_member_with(chat_id, user_id, BanMemberOptions::default())
            .await
    }

    pub async fn ban_member_with(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        options: BanMemberOptions,
    ) -> Result<bool> {
        let request = BanChatMemberRequest::new(chat_id, user_id)
            .with_until_date(options.until_date)
            .with_revoke_messages(options.revoke_messages);
        self.client.chats().ban_chat_member(&request).await
    }

    pub async fn ban_author(&self, message: &Message) -> Result<bool> {
        self.ban_author_with(message, BanMemberOptions::default())
            .await
    }

    pub async fn ban_author_with(
        &self,
        message: &Message,
        options: BanMemberOptions,
    ) -> Result<bool> {
        let user_id = author_user_id(message, "banChatMember")?;
        self.ban_member_with(message.chat.id, user_id, options)
            .await
    }

    pub async fn restrict_member(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        permissions: ChatPermissions,
    ) -> Result<bool> {
        self.restrict_member_with(
            chat_id,
            user_id,
            permissions,
            RestrictMemberOptions::default(),
        )
        .await
    }

    pub async fn restrict_member_with(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        permissions: ChatPermissions,
        options: RestrictMemberOptions,
    ) -> Result<bool> {
        let request = RestrictChatMemberRequest::new(chat_id, user_id, permissions)
            .with_use_independent_chat_permissions(options.use_independent_chat_permissions)
            .with_until_date(options.until_date);
        self.client.chats().restrict_chat_member(&request).await
    }

    pub async fn mute_member(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<bool> {
        self.mute_member_with(chat_id, user_id, RestrictMemberOptions::default())
            .await
    }

    pub async fn mute_member_with(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        options: RestrictMemberOptions,
    ) -> Result<bool> {
        self.restrict_member_with(chat_id, user_id, ChatPermissions::deny_all(), options)
            .await
    }

    pub async fn mute_author(&self, message: &Message) -> Result<bool> {
        self.mute_author_with(message, RestrictMemberOptions::default())
            .await
    }

    pub async fn mute_author_with(
        &self,
        message: &Message,
        options: RestrictMemberOptions,
    ) -> Result<bool> {
        let user_id = author_user_id(message, "restrictChatMember")?;
        self.mute_member_with(message.chat.id, user_id, options)
            .await
    }

    pub async fn delete_message(
        &self,
        chat_id: impl Into<ChatId>,
        message_id: impl Into<MessageId>,
    ) -> Result<bool> {
        let request = DeleteMessageRequest::new(chat_id, message_id.into());
        self.client.messages().delete_message(&request).await
    }

    pub async fn delete(&self, message: &Message) -> Result<bool> {
        self.delete_message(message.chat.id, message.message_id)
            .await
    }

    pub async fn delete_from_update(&self, update: &Update) -> Result<bool> {
        let Some(message) = update_message(update) else {
            return Err(invalid_request(
                "update does not contain a message for deleteMessage",
            ));
        };
        self.delete(message).await
    }
}

/// Blocking app-facing moderation/admin facade.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingModerationApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingModerationApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    pub fn approve_join_request(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<bool> {
        let request = AdvancedApproveChatJoinRequest::new(chat_id, user_id.into());
        self.client
            .advanced()
            .approve_chat_join_request_typed(&request)
    }

    pub fn approve_join_request_from_update(&self, update: &Update) -> Result<bool> {
        let (chat_id, user_id) = join_request_ids(update, "approveChatJoinRequest")?;
        self.approve_join_request(chat_id, user_id)
    }

    pub fn decline_join_request(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<bool> {
        let request = AdvancedDeclineChatJoinRequest::new(chat_id, user_id.into());
        self.client
            .advanced()
            .decline_chat_join_request_typed(&request)
    }

    pub fn decline_join_request_from_update(&self, update: &Update) -> Result<bool> {
        let (chat_id, user_id) = join_request_ids(update, "declineChatJoinRequest")?;
        self.decline_join_request(chat_id, user_id)
    }

    pub fn ban_member(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<bool> {
        self.ban_member_with(chat_id, user_id, BanMemberOptions::default())
    }

    pub fn ban_member_with(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        options: BanMemberOptions,
    ) -> Result<bool> {
        let request = BanChatMemberRequest::new(chat_id, user_id)
            .with_until_date(options.until_date)
            .with_revoke_messages(options.revoke_messages);
        self.client.chats().ban_chat_member(&request)
    }

    pub fn ban_author(&self, message: &Message) -> Result<bool> {
        self.ban_author_with(message, BanMemberOptions::default())
    }

    pub fn ban_author_with(&self, message: &Message, options: BanMemberOptions) -> Result<bool> {
        let user_id = author_user_id(message, "banChatMember")?;
        self.ban_member_with(message.chat.id, user_id, options)
    }

    pub fn restrict_member(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        permissions: ChatPermissions,
    ) -> Result<bool> {
        self.restrict_member_with(
            chat_id,
            user_id,
            permissions,
            RestrictMemberOptions::default(),
        )
    }

    pub fn restrict_member_with(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        permissions: ChatPermissions,
        options: RestrictMemberOptions,
    ) -> Result<bool> {
        let request = RestrictChatMemberRequest::new(chat_id, user_id, permissions)
            .with_use_independent_chat_permissions(options.use_independent_chat_permissions)
            .with_until_date(options.until_date);
        self.client.chats().restrict_chat_member(&request)
    }

    pub fn mute_member(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<bool> {
        self.mute_member_with(chat_id, user_id, RestrictMemberOptions::default())
    }

    pub fn mute_member_with(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        options: RestrictMemberOptions,
    ) -> Result<bool> {
        self.restrict_member_with(chat_id, user_id, ChatPermissions::deny_all(), options)
    }

    pub fn mute_author(&self, message: &Message) -> Result<bool> {
        self.mute_author_with(message, RestrictMemberOptions::default())
    }

    pub fn mute_author_with(
        &self,
        message: &Message,
        options: RestrictMemberOptions,
    ) -> Result<bool> {
        let user_id = author_user_id(message, "restrictChatMember")?;
        self.mute_member_with(message.chat.id, user_id, options)
    }

    pub fn delete_message(
        &self,
        chat_id: impl Into<ChatId>,
        message_id: impl Into<MessageId>,
    ) -> Result<bool> {
        let request = DeleteMessageRequest::new(chat_id, message_id.into());
        self.client.messages().delete_message(&request)
    }

    pub fn delete(&self, message: &Message) -> Result<bool> {
        self.delete_message(message.chat.id, message.message_id)
    }

    pub fn delete_from_update(&self, update: &Update) -> Result<bool> {
        let Some(message) = update_message(update) else {
            return Err(invalid_request(
                "update does not contain a message for deleteMessage",
            ));
        };
        self.delete(message)
    }
}
