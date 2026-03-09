use super::*;

fn get_chat_member_request(
    chat_id: impl Into<ChatId>,
    user_id: impl Into<UserId>,
) -> GetChatMemberRequest {
    GetChatMemberRequest {
        chat_id: chat_id.into(),
        user_id: user_id.into(),
    }
}

fn get_chat_administrators_request(chat_id: impl Into<ChatId>) -> GetChatAdministratorsRequest {
    GetChatAdministratorsRequest {
        chat_id: chat_id.into(),
    }
}

fn missing_capabilities(
    member: &ChatMember,
    capabilities: &[ChatAdministratorCapability],
) -> Vec<ChatAdministratorCapability> {
    capabilities
        .iter()
        .copied()
        .filter(|capability| !member.has_capability(*capability))
        .collect()
}

/// Runtime helper for membership and administrator-capability checks.
///
/// Prefer this from [`AppApi::membership`](crate::client::AppApi::membership) when bot product
/// flows need to verify installation state, administrator rights, or capability prerequisites
/// before enabling a feature.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct MembershipApi {
    client: Client,
}

#[cfg(feature = "_async")]
impl MembershipApi {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Fetches the bot's own user identity via `getMe`.
    pub async fn bot_user(&self) -> Result<User> {
        self.client.bot().get_me().await
    }

    /// Fetches chat administrators for a target chat.
    pub async fn administrators(&self, chat_id: impl Into<ChatId>) -> Result<Vec<ChatMember>> {
        let request = get_chat_administrators_request(chat_id);
        self.client.chats().get_chat_administrators(&request).await
    }

    /// Fetches a concrete member state for `user_id` in `chat_id`.
    pub async fn member(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<ChatMember> {
        let request = get_chat_member_request(chat_id, user_id);
        self.client.chats().get_chat_member(&request).await
    }

    /// Fetches the bot's own member state in a target chat.
    pub async fn bot_member(&self, chat_id: impl Into<ChatId>) -> Result<ChatMember> {
        let chat_id = chat_id.into();
        let bot_user = self.bot_user().await?;
        self.member(chat_id, bot_user.id).await
    }

    /// Returns which administrator capabilities are missing for a member.
    ///
    /// This is useful when install/bind flows want a user-facing list of missing requirements
    /// instead of a simple boolean.
    pub async fn member_missing_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<Vec<ChatAdministratorCapability>> {
        let member = self.member(chat_id, user_id).await?;
        Ok(missing_capabilities(&member, capabilities))
    }

    /// Returns `true` when the member has every required administrator capability.
    pub async fn member_has_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<bool> {
        Ok(self
            .member_missing_capabilities(chat_id, user_id, capabilities)
            .await?
            .is_empty())
    }

    /// Returns which administrator capabilities are missing for the bot in a target chat.
    pub async fn bot_missing_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<Vec<ChatAdministratorCapability>> {
        let member = self.bot_member(chat_id).await?;
        Ok(missing_capabilities(&member, capabilities))
    }

    /// Returns `true` when the bot has every required administrator capability.
    pub async fn bot_has_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<bool> {
        Ok(self
            .bot_missing_capabilities(chat_id, capabilities)
            .await?
            .is_empty())
    }
}

/// Blocking runtime helper for membership and administrator-capability checks.
///
/// Blocking mirror of [`MembershipApi`].
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingMembershipApi {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingMembershipApi {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Fetches the bot's own user identity via `getMe`.
    pub fn bot_user(&self) -> Result<User> {
        self.client.bot().get_me()
    }

    /// Fetches chat administrators for a target chat.
    pub fn administrators(&self, chat_id: impl Into<ChatId>) -> Result<Vec<ChatMember>> {
        let request = get_chat_administrators_request(chat_id);
        self.client.chats().get_chat_administrators(&request)
    }

    /// Fetches a concrete member state for `user_id` in `chat_id`.
    pub fn member(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<ChatMember> {
        let request = get_chat_member_request(chat_id, user_id);
        self.client.chats().get_chat_member(&request)
    }

    /// Fetches the bot's own member state in a target chat.
    pub fn bot_member(&self, chat_id: impl Into<ChatId>) -> Result<ChatMember> {
        let chat_id = chat_id.into();
        let bot_user = self.bot_user()?;
        self.member(chat_id, bot_user.id)
    }

    /// Returns which administrator capabilities are missing for a member.
    pub fn member_missing_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<Vec<ChatAdministratorCapability>> {
        let member = self.member(chat_id, user_id)?;
        Ok(missing_capabilities(&member, capabilities))
    }

    /// Returns `true` when the member has every required administrator capability.
    pub fn member_has_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<bool> {
        Ok(self
            .member_missing_capabilities(chat_id, user_id, capabilities)?
            .is_empty())
    }

    /// Returns which administrator capabilities are missing for the bot in a target chat.
    pub fn bot_missing_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<Vec<ChatAdministratorCapability>> {
        let member = self.bot_member(chat_id)?;
        Ok(missing_capabilities(&member, capabilities))
    }

    /// Returns `true` when the bot has every required administrator capability.
    pub fn bot_has_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<bool> {
        Ok(self
            .bot_missing_capabilities(chat_id, capabilities)?
            .is_empty())
    }
}
