use crate::Result;
use crate::types::chat::{
    BanChatMemberRequest, BanChatSenderChatRequest, ChatInviteLink, ChatMember,
    CreateChatInviteLinkRequest, DeleteChatPhotoRequest, DeleteChatStickerSetRequest,
    EditChatInviteLinkRequest, ExportChatInviteLinkRequest, GetChatAdministratorsRequest,
    GetChatMemberCountRequest, GetChatMemberRequest, GetChatRequest, LeaveChatRequest,
    PinChatMessageRequest, PromoteChatMemberRequest, RestrictChatMemberRequest,
    RevokeChatInviteLinkRequest, SetChatAdministratorCustomTitleRequest, SetChatDescriptionRequest,
    SetChatPermissionsRequest, SetChatStickerSetRequest, SetChatTitleRequest,
    UnbanChatMemberRequest, UnbanChatSenderChatRequest, UnpinAllChatMessagesRequest,
    UnpinChatMessageRequest,
};
use crate::types::message::Chat;

#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;

/// Chat management related methods.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct ChatsService {
    client: Client,
}

#[cfg(feature = "_async")]
impl ChatsService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn get_chat(&self, request: &GetChatRequest) -> Result<Chat> {
        self.client.call_method("getChat", request).await
    }

    pub async fn get_chat_administrators(
        &self,
        request: &GetChatAdministratorsRequest,
    ) -> Result<Vec<ChatMember>> {
        self.client
            .call_method("getChatAdministrators", request)
            .await
    }

    pub async fn get_chat_member_count(&self, request: &GetChatMemberCountRequest) -> Result<u64> {
        self.client.call_method("getChatMemberCount", request).await
    }

    pub async fn get_chat_member(&self, request: &GetChatMemberRequest) -> Result<ChatMember> {
        self.client.call_method("getChatMember", request).await
    }

    pub async fn leave_chat(&self, request: &LeaveChatRequest) -> Result<bool> {
        self.client.call_method("leaveChat", request).await
    }

    pub async fn ban_chat_member(&self, request: &BanChatMemberRequest) -> Result<bool> {
        self.client.call_method("banChatMember", request).await
    }

    pub async fn unban_chat_member(&self, request: &UnbanChatMemberRequest) -> Result<bool> {
        self.client.call_method("unbanChatMember", request).await
    }

    pub async fn restrict_chat_member(&self, request: &RestrictChatMemberRequest) -> Result<bool> {
        self.client.call_method("restrictChatMember", request).await
    }

    pub async fn promote_chat_member(&self, request: &PromoteChatMemberRequest) -> Result<bool> {
        self.client.call_method("promoteChatMember", request).await
    }

    pub async fn set_chat_administrator_custom_title(
        &self,
        request: &SetChatAdministratorCustomTitleRequest,
    ) -> Result<bool> {
        self.client
            .call_method("setChatAdministratorCustomTitle", request)
            .await
    }

    pub async fn ban_chat_sender_chat(&self, request: &BanChatSenderChatRequest) -> Result<bool> {
        self.client.call_method("banChatSenderChat", request).await
    }

    pub async fn unban_chat_sender_chat(
        &self,
        request: &UnbanChatSenderChatRequest,
    ) -> Result<bool> {
        self.client
            .call_method("unbanChatSenderChat", request)
            .await
    }

    pub async fn set_chat_permissions(&self, request: &SetChatPermissionsRequest) -> Result<bool> {
        self.client.call_method("setChatPermissions", request).await
    }

    pub async fn export_chat_invite_link(
        &self,
        request: &ExportChatInviteLinkRequest,
    ) -> Result<String> {
        self.client
            .call_method("exportChatInviteLink", request)
            .await
    }

    pub async fn create_chat_invite_link(
        &self,
        request: &CreateChatInviteLinkRequest,
    ) -> Result<ChatInviteLink> {
        self.client
            .call_method("createChatInviteLink", request)
            .await
    }

    pub async fn edit_chat_invite_link(
        &self,
        request: &EditChatInviteLinkRequest,
    ) -> Result<ChatInviteLink> {
        self.client.call_method("editChatInviteLink", request).await
    }

    pub async fn revoke_chat_invite_link(
        &self,
        request: &RevokeChatInviteLinkRequest,
    ) -> Result<ChatInviteLink> {
        self.client
            .call_method("revokeChatInviteLink", request)
            .await
    }

    pub async fn set_chat_title(&self, request: &SetChatTitleRequest) -> Result<bool> {
        self.client.call_method("setChatTitle", request).await
    }

    pub async fn set_chat_description(&self, request: &SetChatDescriptionRequest) -> Result<bool> {
        self.client.call_method("setChatDescription", request).await
    }

    pub async fn pin_chat_message(&self, request: &PinChatMessageRequest) -> Result<bool> {
        self.client.call_method("pinChatMessage", request).await
    }

    pub async fn unpin_chat_message(&self, request: &UnpinChatMessageRequest) -> Result<bool> {
        self.client.call_method("unpinChatMessage", request).await
    }

    pub async fn unpin_all_chat_messages(
        &self,
        request: &UnpinAllChatMessagesRequest,
    ) -> Result<bool> {
        self.client
            .call_method("unpinAllChatMessages", request)
            .await
    }

    pub async fn delete_chat_photo(&self, request: &DeleteChatPhotoRequest) -> Result<bool> {
        self.client.call_method("deleteChatPhoto", request).await
    }

    pub async fn set_chat_sticker_set(&self, request: &SetChatStickerSetRequest) -> Result<bool> {
        self.client.call_method("setChatStickerSet", request).await
    }

    pub async fn delete_chat_sticker_set(
        &self,
        request: &DeleteChatStickerSetRequest,
    ) -> Result<bool> {
        self.client
            .call_method("deleteChatStickerSet", request)
            .await
    }
}

/// Blocking chat management methods.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingChatsService {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingChatsService {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    pub fn get_chat(&self, request: &GetChatRequest) -> Result<Chat> {
        self.client.call_method("getChat", request)
    }

    pub fn get_chat_administrators(
        &self,
        request: &GetChatAdministratorsRequest,
    ) -> Result<Vec<ChatMember>> {
        self.client.call_method("getChatAdministrators", request)
    }

    pub fn get_chat_member_count(&self, request: &GetChatMemberCountRequest) -> Result<u64> {
        self.client.call_method("getChatMemberCount", request)
    }

    pub fn get_chat_member(&self, request: &GetChatMemberRequest) -> Result<ChatMember> {
        self.client.call_method("getChatMember", request)
    }

    pub fn leave_chat(&self, request: &LeaveChatRequest) -> Result<bool> {
        self.client.call_method("leaveChat", request)
    }

    pub fn ban_chat_member(&self, request: &BanChatMemberRequest) -> Result<bool> {
        self.client.call_method("banChatMember", request)
    }

    pub fn unban_chat_member(&self, request: &UnbanChatMemberRequest) -> Result<bool> {
        self.client.call_method("unbanChatMember", request)
    }

    pub fn restrict_chat_member(&self, request: &RestrictChatMemberRequest) -> Result<bool> {
        self.client.call_method("restrictChatMember", request)
    }

    pub fn promote_chat_member(&self, request: &PromoteChatMemberRequest) -> Result<bool> {
        self.client.call_method("promoteChatMember", request)
    }

    pub fn set_chat_administrator_custom_title(
        &self,
        request: &SetChatAdministratorCustomTitleRequest,
    ) -> Result<bool> {
        self.client
            .call_method("setChatAdministratorCustomTitle", request)
    }

    pub fn ban_chat_sender_chat(&self, request: &BanChatSenderChatRequest) -> Result<bool> {
        self.client.call_method("banChatSenderChat", request)
    }

    pub fn unban_chat_sender_chat(&self, request: &UnbanChatSenderChatRequest) -> Result<bool> {
        self.client.call_method("unbanChatSenderChat", request)
    }

    pub fn set_chat_permissions(&self, request: &SetChatPermissionsRequest) -> Result<bool> {
        self.client.call_method("setChatPermissions", request)
    }

    pub fn export_chat_invite_link(&self, request: &ExportChatInviteLinkRequest) -> Result<String> {
        self.client.call_method("exportChatInviteLink", request)
    }

    pub fn create_chat_invite_link(
        &self,
        request: &CreateChatInviteLinkRequest,
    ) -> Result<ChatInviteLink> {
        self.client.call_method("createChatInviteLink", request)
    }

    pub fn edit_chat_invite_link(
        &self,
        request: &EditChatInviteLinkRequest,
    ) -> Result<ChatInviteLink> {
        self.client.call_method("editChatInviteLink", request)
    }

    pub fn revoke_chat_invite_link(
        &self,
        request: &RevokeChatInviteLinkRequest,
    ) -> Result<ChatInviteLink> {
        self.client.call_method("revokeChatInviteLink", request)
    }

    pub fn set_chat_title(&self, request: &SetChatTitleRequest) -> Result<bool> {
        self.client.call_method("setChatTitle", request)
    }

    pub fn set_chat_description(&self, request: &SetChatDescriptionRequest) -> Result<bool> {
        self.client.call_method("setChatDescription", request)
    }

    pub fn pin_chat_message(&self, request: &PinChatMessageRequest) -> Result<bool> {
        self.client.call_method("pinChatMessage", request)
    }

    pub fn unpin_chat_message(&self, request: &UnpinChatMessageRequest) -> Result<bool> {
        self.client.call_method("unpinChatMessage", request)
    }

    pub fn unpin_all_chat_messages(&self, request: &UnpinAllChatMessagesRequest) -> Result<bool> {
        self.client.call_method("unpinAllChatMessages", request)
    }

    pub fn delete_chat_photo(&self, request: &DeleteChatPhotoRequest) -> Result<bool> {
        self.client.call_method("deleteChatPhoto", request)
    }

    pub fn set_chat_sticker_set(&self, request: &SetChatStickerSetRequest) -> Result<bool> {
        self.client.call_method("setChatStickerSet", request)
    }

    pub fn delete_chat_sticker_set(&self, request: &DeleteChatStickerSetRequest) -> Result<bool> {
        self.client.call_method("deleteChatStickerSet", request)
    }
}
