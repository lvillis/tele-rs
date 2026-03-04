use crate::Result;
use crate::types::bot::{GetUserProfilePhotosRequest, User, UserProfilePhotos};
use crate::types::command::{
    BotCommand, BotDescription, BotName, BotShortDescription, DeleteMyCommandsRequest,
    GetMyCommandsRequest, GetMyDescriptionRequest, GetMyNameRequest, GetMyShortDescriptionRequest,
    SetMyCommandsRequest, SetMyDescriptionRequest, SetMyNameRequest, SetMyShortDescriptionRequest,
};

#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;

/// Bot/account related methods.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct BotService {
    client: Client,
}

#[cfg(feature = "_async")]
impl BotService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls `getMe`.
    pub async fn get_me(&self) -> Result<User> {
        self.client.call_method_no_params("getMe").await
    }

    /// Calls `logOut`.
    pub async fn log_out(&self) -> Result<bool> {
        self.client.call_method_no_params("logOut").await
    }

    /// Calls `close`.
    pub async fn close(&self) -> Result<bool> {
        self.client.call_method_no_params("close").await
    }

    /// Calls `getUserProfilePhotos`.
    pub async fn get_user_profile_photos(
        &self,
        request: &GetUserProfilePhotosRequest,
    ) -> Result<UserProfilePhotos> {
        self.client
            .call_method("getUserProfilePhotos", request)
            .await
    }

    /// Calls `setMyCommands`.
    pub async fn set_my_commands(&self, request: &SetMyCommandsRequest) -> Result<bool> {
        self.client.call_method("setMyCommands", request).await
    }

    /// Calls `getMyCommands`.
    pub async fn get_my_commands(&self, request: &GetMyCommandsRequest) -> Result<Vec<BotCommand>> {
        self.client.call_method("getMyCommands", request).await
    }

    /// Calls `deleteMyCommands`.
    pub async fn delete_my_commands(&self, request: &DeleteMyCommandsRequest) -> Result<bool> {
        self.client.call_method("deleteMyCommands", request).await
    }

    /// Calls `setMyName`.
    pub async fn set_my_name(&self, request: &SetMyNameRequest) -> Result<bool> {
        self.client.call_method("setMyName", request).await
    }

    /// Calls `getMyName`.
    pub async fn get_my_name(&self, request: &GetMyNameRequest) -> Result<BotName> {
        self.client.call_method("getMyName", request).await
    }

    /// Calls `setMyDescription`.
    pub async fn set_my_description(&self, request: &SetMyDescriptionRequest) -> Result<bool> {
        self.client.call_method("setMyDescription", request).await
    }

    /// Calls `getMyDescription`.
    pub async fn get_my_description(
        &self,
        request: &GetMyDescriptionRequest,
    ) -> Result<BotDescription> {
        self.client.call_method("getMyDescription", request).await
    }

    /// Calls `setMyShortDescription`.
    pub async fn set_my_short_description(
        &self,
        request: &SetMyShortDescriptionRequest,
    ) -> Result<bool> {
        self.client
            .call_method("setMyShortDescription", request)
            .await
    }

    /// Calls `getMyShortDescription`.
    pub async fn get_my_short_description(
        &self,
        request: &GetMyShortDescriptionRequest,
    ) -> Result<BotShortDescription> {
        self.client
            .call_method("getMyShortDescription", request)
            .await
    }
}

/// Blocking bot/account methods.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingBotService {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingBotService {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls `getMe`.
    pub fn get_me(&self) -> Result<User> {
        self.client.call_method_no_params("getMe")
    }

    /// Calls `logOut`.
    pub fn log_out(&self) -> Result<bool> {
        self.client.call_method_no_params("logOut")
    }

    /// Calls `close`.
    pub fn close(&self) -> Result<bool> {
        self.client.call_method_no_params("close")
    }

    /// Calls `getUserProfilePhotos`.
    pub fn get_user_profile_photos(
        &self,
        request: &GetUserProfilePhotosRequest,
    ) -> Result<UserProfilePhotos> {
        self.client.call_method("getUserProfilePhotos", request)
    }

    /// Calls `setMyCommands`.
    pub fn set_my_commands(&self, request: &SetMyCommandsRequest) -> Result<bool> {
        self.client.call_method("setMyCommands", request)
    }

    /// Calls `getMyCommands`.
    pub fn get_my_commands(&self, request: &GetMyCommandsRequest) -> Result<Vec<BotCommand>> {
        self.client.call_method("getMyCommands", request)
    }

    /// Calls `deleteMyCommands`.
    pub fn delete_my_commands(&self, request: &DeleteMyCommandsRequest) -> Result<bool> {
        self.client.call_method("deleteMyCommands", request)
    }

    /// Calls `setMyName`.
    pub fn set_my_name(&self, request: &SetMyNameRequest) -> Result<bool> {
        self.client.call_method("setMyName", request)
    }

    /// Calls `getMyName`.
    pub fn get_my_name(&self, request: &GetMyNameRequest) -> Result<BotName> {
        self.client.call_method("getMyName", request)
    }

    /// Calls `setMyDescription`.
    pub fn set_my_description(&self, request: &SetMyDescriptionRequest) -> Result<bool> {
        self.client.call_method("setMyDescription", request)
    }

    /// Calls `getMyDescription`.
    pub fn get_my_description(&self, request: &GetMyDescriptionRequest) -> Result<BotDescription> {
        self.client.call_method("getMyDescription", request)
    }

    /// Calls `setMyShortDescription`.
    pub fn set_my_short_description(&self, request: &SetMyShortDescriptionRequest) -> Result<bool> {
        self.client.call_method("setMyShortDescription", request)
    }

    /// Calls `getMyShortDescription`.
    pub fn get_my_short_description(
        &self,
        request: &GetMyShortDescriptionRequest,
    ) -> Result<BotShortDescription> {
        self.client.call_method("getMyShortDescription", request)
    }
}
