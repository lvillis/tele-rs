use crate::Result;
use crate::types::file::File;
use crate::types::message::Message;
use crate::types::sticker::{
    AddStickerToSetRequest, CreateNewStickerSetRequest, DeleteStickerFromSetRequest,
    DeleteStickerSetRequest, GetCustomEmojiStickersRequest, GetStickerSetRequest,
    ReplaceStickerInSetRequest, SendStickerRequest, SetCustomEmojiStickerSetThumbnailRequest,
    SetStickerEmojiListRequest, SetStickerKeywordsRequest, SetStickerMaskPositionRequest,
    SetStickerPositionInSetRequest, SetStickerSetThumbnailRequest, SetStickerSetTitleRequest,
    Sticker, StickerSet, UploadStickerFileRequest,
};
use crate::types::upload::UploadFile;

#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;

/// Sticker and sticker set methods.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct StickersService {
    client: Client,
}

#[cfg(feature = "_async")]
impl StickersService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls `sendSticker`.
    pub async fn send_sticker(&self, request: &SendStickerRequest) -> Result<Message> {
        request.validate()?;
        self.client.call_method("sendSticker", request).await
    }

    /// Calls `sendSticker` using multipart upload for local bytes.
    /// `request.sticker` is ignored; file content is taken from `file`.
    pub async fn send_sticker_upload(
        &self,
        request: &SendStickerRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        request.validate()?;
        self.client
            .call_method_multipart("sendSticker", request, "sticker", file)
            .await
    }

    /// Calls `getStickerSet`.
    pub async fn get_sticker_set(&self, request: &GetStickerSetRequest) -> Result<StickerSet> {
        request.validate()?;
        self.client.call_method("getStickerSet", request).await
    }

    /// Calls `getCustomEmojiStickers`.
    pub async fn get_custom_emoji_stickers(
        &self,
        request: &GetCustomEmojiStickersRequest,
    ) -> Result<Vec<Sticker>> {
        request.validate()?;
        self.client
            .call_method("getCustomEmojiStickers", request)
            .await
    }

    /// Calls `uploadStickerFile`.
    pub async fn upload_sticker_file(&self, request: &UploadStickerFileRequest) -> Result<File> {
        request.validate()?;
        self.client.call_method("uploadStickerFile", request).await
    }

    /// Calls `uploadStickerFile` using multipart upload for local bytes.
    /// `request.sticker` is ignored; file content is taken from `file`.
    pub async fn upload_sticker_file_upload(
        &self,
        request: &UploadStickerFileRequest,
        file: &UploadFile,
    ) -> Result<File> {
        request.validate()?;
        self.client
            .call_method_multipart("uploadStickerFile", request, "sticker", file)
            .await
    }

    /// Calls `createNewStickerSet`.
    pub async fn create_new_sticker_set(
        &self,
        request: &CreateNewStickerSetRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("createNewStickerSet", request)
            .await
    }

    /// Calls `addStickerToSet`.
    pub async fn add_sticker_to_set(&self, request: &AddStickerToSetRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("addStickerToSet", request).await
    }

    /// Calls `setStickerPositionInSet`.
    pub async fn set_sticker_position_in_set(
        &self,
        request: &SetStickerPositionInSetRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("setStickerPositionInSet", request)
            .await
    }

    /// Calls `deleteStickerFromSet`.
    pub async fn delete_sticker_from_set(
        &self,
        request: &DeleteStickerFromSetRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("deleteStickerFromSet", request)
            .await
    }

    /// Calls `replaceStickerInSet`.
    pub async fn replace_sticker_in_set(
        &self,
        request: &ReplaceStickerInSetRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("replaceStickerInSet", request)
            .await
    }

    /// Calls `setStickerEmojiList`.
    pub async fn set_sticker_emoji_list(
        &self,
        request: &SetStickerEmojiListRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("setStickerEmojiList", request)
            .await
    }

    /// Calls `setStickerKeywords`.
    pub async fn set_sticker_keywords(&self, request: &SetStickerKeywordsRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("setStickerKeywords", request).await
    }

    /// Calls `setStickerMaskPosition`.
    pub async fn set_sticker_mask_position(
        &self,
        request: &SetStickerMaskPositionRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("setStickerMaskPosition", request)
            .await
    }

    /// Calls `setStickerSetTitle`.
    pub async fn set_sticker_set_title(&self, request: &SetStickerSetTitleRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("setStickerSetTitle", request).await
    }

    /// Calls `setStickerSetThumbnail`.
    pub async fn set_sticker_set_thumbnail(
        &self,
        request: &SetStickerSetThumbnailRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("setStickerSetThumbnail", request)
            .await
    }

    /// Calls `setStickerSetThumbnail` using multipart upload for local bytes.
    /// `request.thumbnail` is ignored; file content is taken from `file`.
    pub async fn set_sticker_set_thumbnail_upload(
        &self,
        request: &SetStickerSetThumbnailRequest,
        file: &UploadFile,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method_multipart("setStickerSetThumbnail", request, "thumbnail", file)
            .await
    }

    /// Calls `setCustomEmojiStickerSetThumbnail`.
    pub async fn set_custom_emoji_sticker_set_thumbnail(
        &self,
        request: &SetCustomEmojiStickerSetThumbnailRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("setCustomEmojiStickerSetThumbnail", request)
            .await
    }

    /// Calls `deleteStickerSet`.
    pub async fn delete_sticker_set(&self, request: &DeleteStickerSetRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("deleteStickerSet", request).await
    }
}

/// Blocking sticker methods.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingStickersService {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingStickersService {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls `sendSticker`.
    pub fn send_sticker(&self, request: &SendStickerRequest) -> Result<Message> {
        request.validate()?;
        self.client.call_method("sendSticker", request)
    }

    /// Calls `sendSticker` using multipart upload for local bytes.
    /// `request.sticker` is ignored; file content is taken from `file`.
    pub fn send_sticker_upload(
        &self,
        request: &SendStickerRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        request.validate()?;
        self.client
            .call_method_multipart("sendSticker", request, "sticker", file)
    }

    /// Calls `getStickerSet`.
    pub fn get_sticker_set(&self, request: &GetStickerSetRequest) -> Result<StickerSet> {
        request.validate()?;
        self.client.call_method("getStickerSet", request)
    }

    /// Calls `getCustomEmojiStickers`.
    pub fn get_custom_emoji_stickers(
        &self,
        request: &GetCustomEmojiStickersRequest,
    ) -> Result<Vec<Sticker>> {
        request.validate()?;
        self.client.call_method("getCustomEmojiStickers", request)
    }

    /// Calls `uploadStickerFile`.
    pub fn upload_sticker_file(&self, request: &UploadStickerFileRequest) -> Result<File> {
        request.validate()?;
        self.client.call_method("uploadStickerFile", request)
    }

    /// Calls `uploadStickerFile` using multipart upload for local bytes.
    /// `request.sticker` is ignored; file content is taken from `file`.
    pub fn upload_sticker_file_upload(
        &self,
        request: &UploadStickerFileRequest,
        file: &UploadFile,
    ) -> Result<File> {
        request.validate()?;
        self.client
            .call_method_multipart("uploadStickerFile", request, "sticker", file)
    }

    /// Calls `createNewStickerSet`.
    pub fn create_new_sticker_set(&self, request: &CreateNewStickerSetRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("createNewStickerSet", request)
    }

    /// Calls `addStickerToSet`.
    pub fn add_sticker_to_set(&self, request: &AddStickerToSetRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("addStickerToSet", request)
    }

    /// Calls `setStickerPositionInSet`.
    pub fn set_sticker_position_in_set(
        &self,
        request: &SetStickerPositionInSetRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client.call_method("setStickerPositionInSet", request)
    }

    /// Calls `deleteStickerFromSet`.
    pub fn delete_sticker_from_set(&self, request: &DeleteStickerFromSetRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("deleteStickerFromSet", request)
    }

    /// Calls `replaceStickerInSet`.
    pub fn replace_sticker_in_set(&self, request: &ReplaceStickerInSetRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("replaceStickerInSet", request)
    }

    /// Calls `setStickerEmojiList`.
    pub fn set_sticker_emoji_list(&self, request: &SetStickerEmojiListRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("setStickerEmojiList", request)
    }

    /// Calls `setStickerKeywords`.
    pub fn set_sticker_keywords(&self, request: &SetStickerKeywordsRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("setStickerKeywords", request)
    }

    /// Calls `setStickerMaskPosition`.
    pub fn set_sticker_mask_position(
        &self,
        request: &SetStickerMaskPositionRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client.call_method("setStickerMaskPosition", request)
    }

    /// Calls `setStickerSetTitle`.
    pub fn set_sticker_set_title(&self, request: &SetStickerSetTitleRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("setStickerSetTitle", request)
    }

    /// Calls `setStickerSetThumbnail`.
    pub fn set_sticker_set_thumbnail(
        &self,
        request: &SetStickerSetThumbnailRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client.call_method("setStickerSetThumbnail", request)
    }

    /// Calls `setStickerSetThumbnail` using multipart upload for local bytes.
    /// `request.thumbnail` is ignored; file content is taken from `file`.
    pub fn set_sticker_set_thumbnail_upload(
        &self,
        request: &SetStickerSetThumbnailRequest,
        file: &UploadFile,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method_multipart("setStickerSetThumbnail", request, "thumbnail", file)
    }

    /// Calls `setCustomEmojiStickerSetThumbnail`.
    pub fn set_custom_emoji_sticker_set_thumbnail(
        &self,
        request: &SetCustomEmojiStickerSetThumbnailRequest,
    ) -> Result<bool> {
        request.validate()?;
        self.client
            .call_method("setCustomEmojiStickerSetThumbnail", request)
    }

    /// Calls `deleteStickerSet`.
    pub fn delete_sticker_set(&self, request: &DeleteStickerSetRequest) -> Result<bool> {
        request.validate()?;
        self.client.call_method("deleteStickerSet", request)
    }
}
