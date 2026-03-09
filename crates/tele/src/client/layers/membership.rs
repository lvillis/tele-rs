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

/// App-facing helper for membership and capability checks commonly used in install/bind flows.
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

    pub async fn bot_user(&self) -> Result<User> {
        self.client.bot().get_me().await
    }

    pub async fn administrators(&self, chat_id: impl Into<ChatId>) -> Result<Vec<ChatMember>> {
        let request = get_chat_administrators_request(chat_id);
        self.client.chats().get_chat_administrators(&request).await
    }

    pub async fn member(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<ChatMember> {
        let request = get_chat_member_request(chat_id, user_id);
        self.client.chats().get_chat_member(&request).await
    }

    pub async fn bot_member(&self, chat_id: impl Into<ChatId>) -> Result<ChatMember> {
        let chat_id = chat_id.into();
        let bot_user = self.bot_user().await?;
        self.member(chat_id, bot_user.id).await
    }

    pub async fn member_missing_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<Vec<ChatAdministratorCapability>> {
        let member = self.member(chat_id, user_id).await?;
        Ok(missing_capabilities(&member, capabilities))
    }

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

    pub async fn bot_missing_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<Vec<ChatAdministratorCapability>> {
        let member = self.bot_member(chat_id).await?;
        Ok(missing_capabilities(&member, capabilities))
    }

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

/// Blocking app-facing helper for membership and capability checks.
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

    pub fn bot_user(&self) -> Result<User> {
        self.client.bot().get_me()
    }

    pub fn administrators(&self, chat_id: impl Into<ChatId>) -> Result<Vec<ChatMember>> {
        let request = get_chat_administrators_request(chat_id);
        self.client.chats().get_chat_administrators(&request)
    }

    pub fn member(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
    ) -> Result<ChatMember> {
        let request = get_chat_member_request(chat_id, user_id);
        self.client.chats().get_chat_member(&request)
    }

    pub fn bot_member(&self, chat_id: impl Into<ChatId>) -> Result<ChatMember> {
        let chat_id = chat_id.into();
        let bot_user = self.bot_user()?;
        self.member(chat_id, bot_user.id)
    }

    pub fn member_missing_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        user_id: impl Into<UserId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<Vec<ChatAdministratorCapability>> {
        let member = self.member(chat_id, user_id)?;
        Ok(missing_capabilities(&member, capabilities))
    }

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

    pub fn bot_missing_capabilities(
        &self,
        chat_id: impl Into<ChatId>,
        capabilities: &[ChatAdministratorCapability],
    ) -> Result<Vec<ChatAdministratorCapability>> {
        let member = self.bot_member(chat_id)?;
        Ok(missing_capabilities(&member, capabilities))
    }

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
