use super::support::{invalid_request, update_message};
#[cfg(feature = "_async")]
use super::{AppApi, TextSendBuilder};
#[cfg(feature = "_blocking")]
use super::{BlockingAppApi, BlockingTextSendBuilder};
#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;
use crate::Result;
use crate::types::advanced::{
    AdvancedApproveChatJoinRequest, AdvancedDeclineChatJoinRequest,
    AdvancedDeleteBusinessMessagesRequest, AdvancedDeleteEphemeralMessageRequest,
};
use crate::types::chat::{BanChatMemberRequest, ChatPermissions, RestrictChatMemberRequest};
use crate::types::common::{ChatId, MessageId, UserId};
use crate::types::message::{DeleteMessageRequest, Message};
use crate::types::update::Update;

fn author_user_id(message: &Message, method: &str) -> Result<UserId> {
    message.sender_user().map(|user| user.id).ok_or_else(|| {
        invalid_request(format!(
            "{method} requires a user sender: from must be present and sender_chat must be absent"
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

enum DeleteTarget {
    Chat(DeleteMessageRequest),
    Business(AdvancedDeleteBusinessMessagesRequest),
    Ephemeral(AdvancedDeleteEphemeralMessageRequest),
}

impl DeleteTarget {
    fn from_message(message: &Message) -> Result<Self> {
        Self::from_message_with_receiver(message, None)
    }

    fn from_message_with_receiver(
        message: &Message,
        callback_receiver: Option<UserId>,
    ) -> Result<Self> {
        if message.guest_query_id.is_some() {
            return Err(invalid_request(
                "guest messages cannot be deleted by this bot",
            ));
        }
        if let Some(ephemeral_message_id) = message.ephemeral_message_id {
            if message.business_connection_id.is_some() {
                return Err(invalid_request(
                    "ephemeral deletion cannot target a business connection",
                ));
            }
            let receiver = message
                .receiver_user
                .as_ref()
                .map(|user| user.id)
                .or(callback_receiver)
                .ok_or_else(|| {
                    invalid_request(
                        "ephemeral deletion requires receiver_user to identify the recipient",
                    )
                })?;
            return Ok(Self::Ephemeral(AdvancedDeleteEphemeralMessageRequest::new(
                message.chat.id,
                receiver,
                ephemeral_message_id.into(),
            )));
        }
        if let Some(business_connection_id) = message.business_connection_id.as_deref() {
            return Ok(Self::Business(AdvancedDeleteBusinessMessagesRequest::new(
                business_connection_id,
                vec![message.message_id],
            )));
        }
        Ok(Self::Chat(DeleteMessageRequest::new(
            message.chat.id,
            message.message_id,
        )))
    }

    fn from_update(update: &Update) -> Result<Self> {
        if update.guest_message.is_some() {
            return Err(invalid_request(
                "guest messages cannot be deleted by this bot",
            ));
        }
        let message = update_message(update).ok_or_else(|| {
            invalid_request("update does not contain an accessible message to delete")
        })?;
        if (update.business_message.is_some() || update.edited_business_message.is_some())
            && message.business_connection_id.is_none()
        {
            return Err(invalid_request(
                "business deletion requires business_connection_id",
            ));
        }
        let callback_receiver = update.callback_query.as_ref().and_then(|query| {
            query
                .message
                .as_deref()
                .and_then(|value| value.accessible())
                .filter(|value| std::ptr::eq(*value, message))
                .map(|_| query.from.id)
        });
        Self::from_message_with_receiver(message, callback_receiver)
    }
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

/// Governance notice facade that keeps moderation-related notifications on the app layer.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct ModerationNoticeApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl ModerationNoticeApi {
    fn new(client: Client) -> Self {
        Self { client }
    }

    /// Starts a governance notice send to a specific chat.
    pub fn text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<TextSendBuilder> {
        AppApi::new(self.client.clone()).text(chat_id, text)
    }

    /// Starts a governance notice send using the canonical reply chat derived from an update.
    pub fn reply(&self, update: &Update, text: impl Into<String>) -> Result<TextSendBuilder> {
        AppApi::new(self.client.clone()).reply(update, text)
    }

    /// Starts a governance notice reply anchored to a specific message.
    pub fn for_message(
        &self,
        message: &Message,
        text: impl Into<String>,
    ) -> Result<TextSendBuilder> {
        AppApi::new(self.client.clone()).reply_to(message, text)
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

    /// Dedicated governance notice facade that reuses the canonical app text builder.
    pub fn notice(&self) -> ModerationNoticeApi {
        ModerationNoticeApi::new(self.client.clone())
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

    /// Bans the user who sent the message.
    ///
    /// Returns an error before sending a request if the sender is unknown or
    /// `sender_chat` is present. Never substitutes a compatibility user or bans a chat.
    pub async fn ban_author(&self, message: &Message) -> Result<bool> {
        self.ban_author_with(message, BanMemberOptions::default())
            .await
    }

    /// Bans the user who sent the message.
    ///
    /// Returns an error before sending a request if the sender is unknown or
    /// `sender_chat` is present. Never substitutes a compatibility user or bans a chat.
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

    /// Mutes the user who sent the message.
    ///
    /// Returns an error before sending a request if the sender is unknown or
    /// `sender_chat` is present. Never substitutes a compatibility user or bans a chat.
    pub async fn mute_author(&self, message: &Message) -> Result<bool> {
        self.mute_author_with(message, RestrictMemberOptions::default())
            .await
    }

    /// Mutes the user who sent the message.
    ///
    /// Returns an error before sending a request if the sender is unknown or
    /// `sender_chat` is present. Never substitutes a compatibility user or bans a chat.
    pub async fn mute_author_with(
        &self,
        message: &Message,
        options: RestrictMemberOptions,
    ) -> Result<bool> {
        let user_id = author_user_id(message, "restrictChatMember")?;
        self.mute_member_with(message.chat.id, user_id, options)
            .await
    }

    /// Deletes a regular bot-chat message by its identifiers.
    /// Use [`Self::delete`] to preserve a message's Business or ephemeral context.
    pub async fn delete_message(
        &self,
        chat_id: impl Into<ChatId>,
        message_id: impl Into<MessageId>,
    ) -> Result<bool> {
        let request = DeleteMessageRequest::new(chat_id, message_id.into());
        self.client.messages().delete_message(&request).await
    }

    /// Deletes a message using its regular, Business, or ephemeral context.
    /// Guest messages and ephemeral messages without `receiver_user` are rejected locally.
    pub async fn delete(&self, message: &Message) -> Result<bool> {
        self.delete_target(DeleteTarget::from_message(message)?)
            .await
    }

    /// Deletes the accessible message in an update, preserving its deletion context.
    /// An ephemeral callback's sender identifies the recipient when `receiver_user` is absent.
    pub async fn delete_from_update(&self, update: &Update) -> Result<bool> {
        self.delete_target(DeleteTarget::from_update(update)?).await
    }

    async fn delete_target(&self, target: DeleteTarget) -> Result<bool> {
        match target {
            DeleteTarget::Chat(request) => self.client.messages().delete_message(&request).await,
            DeleteTarget::Business(request) => {
                self.client
                    .advanced()
                    .delete_business_messages_typed(&request)
                    .await
            }
            DeleteTarget::Ephemeral(request) => {
                self.client
                    .advanced()
                    .delete_ephemeral_message_typed(&request)
                    .await
            }
        }
    }
}

/// Blocking governance notice facade that keeps moderation-related notifications on the app layer.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingModerationNoticeApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingModerationNoticeApi {
    fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    pub fn text(
        &self,
        chat_id: impl Into<ChatId>,
        text: impl Into<String>,
    ) -> Result<BlockingTextSendBuilder> {
        BlockingAppApi::new(self.client.clone()).text(chat_id, text)
    }

    pub fn reply(
        &self,
        update: &Update,
        text: impl Into<String>,
    ) -> Result<BlockingTextSendBuilder> {
        BlockingAppApi::new(self.client.clone()).reply(update, text)
    }

    pub fn for_message(
        &self,
        message: &Message,
        text: impl Into<String>,
    ) -> Result<BlockingTextSendBuilder> {
        BlockingAppApi::new(self.client.clone()).reply_to(message, text)
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

    pub fn notice(&self) -> BlockingModerationNoticeApi {
        BlockingModerationNoticeApi::new(self.client.clone())
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

    /// Bans the user who sent the message.
    ///
    /// Returns an error before sending a request if the sender is unknown or
    /// `sender_chat` is present. Never substitutes a compatibility user or bans a chat.
    pub fn ban_author(&self, message: &Message) -> Result<bool> {
        self.ban_author_with(message, BanMemberOptions::default())
    }

    /// Bans the user who sent the message.
    ///
    /// Returns an error before sending a request if the sender is unknown or
    /// `sender_chat` is present. Never substitutes a compatibility user or bans a chat.
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

    /// Mutes the user who sent the message.
    ///
    /// Returns an error before sending a request if the sender is unknown or
    /// `sender_chat` is present. Never substitutes a compatibility user or bans a chat.
    pub fn mute_author(&self, message: &Message) -> Result<bool> {
        self.mute_author_with(message, RestrictMemberOptions::default())
    }

    /// Mutes the user who sent the message.
    ///
    /// Returns an error before sending a request if the sender is unknown or
    /// `sender_chat` is present. Never substitutes a compatibility user or bans a chat.
    pub fn mute_author_with(
        &self,
        message: &Message,
        options: RestrictMemberOptions,
    ) -> Result<bool> {
        let user_id = author_user_id(message, "restrictChatMember")?;
        self.mute_member_with(message.chat.id, user_id, options)
    }

    /// Deletes a regular bot-chat message by its identifiers.
    /// Use [`Self::delete`] to preserve a message's Business or ephemeral context.
    pub fn delete_message(
        &self,
        chat_id: impl Into<ChatId>,
        message_id: impl Into<MessageId>,
    ) -> Result<bool> {
        let request = DeleteMessageRequest::new(chat_id, message_id.into());
        self.client.messages().delete_message(&request)
    }

    /// Deletes a message using its regular, Business, or ephemeral context.
    /// Guest messages and ephemeral messages without `receiver_user` are rejected locally.
    pub fn delete(&self, message: &Message) -> Result<bool> {
        self.delete_target(DeleteTarget::from_message(message)?)
    }

    /// Deletes the accessible message in an update, preserving its deletion context.
    /// An ephemeral callback's sender identifies the recipient when `receiver_user` is absent.
    pub fn delete_from_update(&self, update: &Update) -> Result<bool> {
        self.delete_target(DeleteTarget::from_update(update)?)
    }

    fn delete_target(&self, target: DeleteTarget) -> Result<bool> {
        match target {
            DeleteTarget::Chat(request) => self.client.messages().delete_message(&request),
            DeleteTarget::Business(request) => self
                .client
                .advanced()
                .delete_business_messages_typed(&request),
            DeleteTarget::Ephemeral(request) => self
                .client
                .advanced()
                .delete_ephemeral_message_typed(&request),
        }
    }
}
