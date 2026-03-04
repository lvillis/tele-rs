use crate::Result;
use crate::types::message::{
    CopyMessageRequest, CopyMessagesRequest, DeleteMessageRequest, DeleteMessagesRequest,
    EditMessageCaptionRequest, EditMessageLiveLocationRequest, EditMessageReplyMarkupRequest,
    EditMessageResult, EditMessageTextRequest, ForwardMessageRequest, Message, MessageIdObject,
    Poll, SendAnimationRequest, SendAudioRequest, SendChatActionRequest, SendContactRequest,
    SendDiceRequest, SendDocumentRequest, SendLocationRequest, SendMediaGroupRequest,
    SendMessageRequest, SendPhotoRequest, SendPollRequest, SendVenueRequest, SendVideoNoteRequest,
    SendVideoRequest, SendVoiceRequest, StopMessageLiveLocationRequest, StopPollRequest,
};
use crate::types::upload::UploadFile;

#[cfg(feature = "_blocking")]
use crate::BlockingClient;
#[cfg(feature = "_async")]
use crate::Client;

/// Message related methods.
#[cfg(feature = "_async")]
#[derive(Clone)]
pub struct MessagesService {
    client: Client,
}

#[cfg(feature = "_async")]
impl MessagesService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls `sendMessage`.
    pub async fn send_message(&self, request: &SendMessageRequest) -> Result<Message> {
        self.client.call_method("sendMessage", request).await
    }

    /// Calls `forwardMessage`.
    pub async fn forward_message(&self, request: &ForwardMessageRequest) -> Result<Message> {
        self.client.call_method("forwardMessage", request).await
    }

    /// Calls `copyMessage`.
    pub async fn copy_message(&self, request: &CopyMessageRequest) -> Result<MessageIdObject> {
        self.client.call_method("copyMessage", request).await
    }

    /// Calls `copyMessages`.
    pub async fn copy_messages(
        &self,
        request: &CopyMessagesRequest,
    ) -> Result<Vec<MessageIdObject>> {
        self.client.call_method("copyMessages", request).await
    }

    /// Calls `sendPhoto`.
    pub async fn send_photo(&self, request: &SendPhotoRequest) -> Result<Message> {
        self.client.call_method("sendPhoto", request).await
    }

    /// Calls `sendPhoto` using multipart upload for local bytes.
    /// `request.photo` is ignored; file content is taken from `file`.
    pub async fn send_photo_upload(
        &self,
        request: &SendPhotoRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendPhoto", request, "photo", file)
            .await
    }

    /// Calls `sendAudio`.
    pub async fn send_audio(&self, request: &SendAudioRequest) -> Result<Message> {
        self.client.call_method("sendAudio", request).await
    }

    /// Calls `sendAudio` using multipart upload for local bytes.
    /// `request.audio` is ignored; file content is taken from `file`.
    pub async fn send_audio_upload(
        &self,
        request: &SendAudioRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendAudio", request, "audio", file)
            .await
    }

    /// Calls `sendDocument`.
    pub async fn send_document(&self, request: &SendDocumentRequest) -> Result<Message> {
        self.client.call_method("sendDocument", request).await
    }

    /// Calls `sendDocument` using multipart upload for local bytes.
    /// `request.document` is ignored; file content is taken from `file`.
    pub async fn send_document_upload(
        &self,
        request: &SendDocumentRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendDocument", request, "document", file)
            .await
    }

    /// Calls `sendVideo`.
    pub async fn send_video(&self, request: &SendVideoRequest) -> Result<Message> {
        self.client.call_method("sendVideo", request).await
    }

    /// Calls `sendVideo` using multipart upload for local bytes.
    /// `request.video` is ignored; file content is taken from `file`.
    pub async fn send_video_upload(
        &self,
        request: &SendVideoRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendVideo", request, "video", file)
            .await
    }

    /// Calls `sendAnimation`.
    pub async fn send_animation(&self, request: &SendAnimationRequest) -> Result<Message> {
        self.client.call_method("sendAnimation", request).await
    }

    /// Calls `sendAnimation` using multipart upload for local bytes.
    /// `request.animation` is ignored; file content is taken from `file`.
    pub async fn send_animation_upload(
        &self,
        request: &SendAnimationRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendAnimation", request, "animation", file)
            .await
    }

    /// Calls `sendVoice`.
    pub async fn send_voice(&self, request: &SendVoiceRequest) -> Result<Message> {
        self.client.call_method("sendVoice", request).await
    }

    /// Calls `sendVoice` using multipart upload for local bytes.
    /// `request.voice` is ignored; file content is taken from `file`.
    pub async fn send_voice_upload(
        &self,
        request: &SendVoiceRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendVoice", request, "voice", file)
            .await
    }

    /// Calls `sendVideoNote`.
    pub async fn send_video_note(&self, request: &SendVideoNoteRequest) -> Result<Message> {
        self.client.call_method("sendVideoNote", request).await
    }

    /// Calls `sendVideoNote` using multipart upload for local bytes.
    /// `request.video_note` is ignored; file content is taken from `file`.
    pub async fn send_video_note_upload(
        &self,
        request: &SendVideoNoteRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendVideoNote", request, "video_note", file)
            .await
    }

    /// Calls `sendMediaGroup`.
    pub async fn send_media_group(&self, request: &SendMediaGroupRequest) -> Result<Vec<Message>> {
        self.client.call_method("sendMediaGroup", request).await
    }

    /// Calls `sendLocation`.
    pub async fn send_location(&self, request: &SendLocationRequest) -> Result<Message> {
        self.client.call_method("sendLocation", request).await
    }

    /// Calls `sendVenue`.
    pub async fn send_venue(&self, request: &SendVenueRequest) -> Result<Message> {
        self.client.call_method("sendVenue", request).await
    }

    /// Calls `sendContact`.
    pub async fn send_contact(&self, request: &SendContactRequest) -> Result<Message> {
        self.client.call_method("sendContact", request).await
    }

    /// Calls `sendPoll`.
    pub async fn send_poll(&self, request: &SendPollRequest) -> Result<Message> {
        self.client.call_method("sendPoll", request).await
    }

    /// Calls `stopPoll`.
    pub async fn stop_poll(&self, request: &StopPollRequest) -> Result<Poll> {
        self.client.call_method("stopPoll", request).await
    }

    /// Calls `sendDice`.
    pub async fn send_dice(&self, request: &SendDiceRequest) -> Result<Message> {
        self.client.call_method("sendDice", request).await
    }

    /// Calls `sendChatAction`.
    pub async fn send_chat_action(&self, request: &SendChatActionRequest) -> Result<bool> {
        self.client.call_method("sendChatAction", request).await
    }

    /// Calls `editMessageText`.
    pub async fn edit_message_text(
        &self,
        request: &EditMessageTextRequest,
    ) -> Result<EditMessageResult> {
        request.validate()?;
        self.client.call_method("editMessageText", request).await
    }

    /// Calls `editMessageCaption`.
    pub async fn edit_message_caption(
        &self,
        request: &EditMessageCaptionRequest,
    ) -> Result<EditMessageResult> {
        request.validate()?;
        self.client.call_method("editMessageCaption", request).await
    }

    /// Calls `editMessageReplyMarkup`.
    pub async fn edit_message_reply_markup(
        &self,
        request: &EditMessageReplyMarkupRequest,
    ) -> Result<EditMessageResult> {
        request.validate()?;
        self.client
            .call_method("editMessageReplyMarkup", request)
            .await
    }

    /// Calls `editMessageLiveLocation`.
    pub async fn edit_message_live_location(
        &self,
        request: &EditMessageLiveLocationRequest,
    ) -> Result<EditMessageResult> {
        request.validate()?;
        self.client
            .call_method("editMessageLiveLocation", request)
            .await
    }

    /// Calls `stopMessageLiveLocation`.
    pub async fn stop_message_live_location(
        &self,
        request: &StopMessageLiveLocationRequest,
    ) -> Result<EditMessageResult> {
        request.validate()?;
        self.client
            .call_method("stopMessageLiveLocation", request)
            .await
    }

    /// Calls `deleteMessage`.
    pub async fn delete_message(&self, request: &DeleteMessageRequest) -> Result<bool> {
        self.client.call_method("deleteMessage", request).await
    }

    /// Calls `deleteMessages`.
    pub async fn delete_messages(&self, request: &DeleteMessagesRequest) -> Result<bool> {
        self.client.call_method("deleteMessages", request).await
    }
}

/// Blocking message methods.
#[cfg(feature = "_blocking")]
#[derive(Clone)]
pub struct BlockingMessagesService {
    client: BlockingClient,
}

#[cfg(feature = "_blocking")]
impl BlockingMessagesService {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls `sendMessage`.
    pub fn send_message(&self, request: &SendMessageRequest) -> Result<Message> {
        self.client.call_method("sendMessage", request)
    }

    /// Calls `forwardMessage`.
    pub fn forward_message(&self, request: &ForwardMessageRequest) -> Result<Message> {
        self.client.call_method("forwardMessage", request)
    }

    /// Calls `copyMessage`.
    pub fn copy_message(&self, request: &CopyMessageRequest) -> Result<MessageIdObject> {
        self.client.call_method("copyMessage", request)
    }

    /// Calls `copyMessages`.
    pub fn copy_messages(&self, request: &CopyMessagesRequest) -> Result<Vec<MessageIdObject>> {
        self.client.call_method("copyMessages", request)
    }

    /// Calls `sendPhoto`.
    pub fn send_photo(&self, request: &SendPhotoRequest) -> Result<Message> {
        self.client.call_method("sendPhoto", request)
    }

    /// Calls `sendPhoto` using multipart upload for local bytes.
    /// `request.photo` is ignored; file content is taken from `file`.
    pub fn send_photo_upload(
        &self,
        request: &SendPhotoRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendPhoto", request, "photo", file)
    }

    /// Calls `sendAudio`.
    pub fn send_audio(&self, request: &SendAudioRequest) -> Result<Message> {
        self.client.call_method("sendAudio", request)
    }

    /// Calls `sendAudio` using multipart upload for local bytes.
    /// `request.audio` is ignored; file content is taken from `file`.
    pub fn send_audio_upload(
        &self,
        request: &SendAudioRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendAudio", request, "audio", file)
    }

    /// Calls `sendDocument`.
    pub fn send_document(&self, request: &SendDocumentRequest) -> Result<Message> {
        self.client.call_method("sendDocument", request)
    }

    /// Calls `sendDocument` using multipart upload for local bytes.
    /// `request.document` is ignored; file content is taken from `file`.
    pub fn send_document_upload(
        &self,
        request: &SendDocumentRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendDocument", request, "document", file)
    }

    /// Calls `sendVideo`.
    pub fn send_video(&self, request: &SendVideoRequest) -> Result<Message> {
        self.client.call_method("sendVideo", request)
    }

    /// Calls `sendVideo` using multipart upload for local bytes.
    /// `request.video` is ignored; file content is taken from `file`.
    pub fn send_video_upload(
        &self,
        request: &SendVideoRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendVideo", request, "video", file)
    }

    /// Calls `sendAnimation`.
    pub fn send_animation(&self, request: &SendAnimationRequest) -> Result<Message> {
        self.client.call_method("sendAnimation", request)
    }

    /// Calls `sendAnimation` using multipart upload for local bytes.
    /// `request.animation` is ignored; file content is taken from `file`.
    pub fn send_animation_upload(
        &self,
        request: &SendAnimationRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendAnimation", request, "animation", file)
    }

    /// Calls `sendVoice`.
    pub fn send_voice(&self, request: &SendVoiceRequest) -> Result<Message> {
        self.client.call_method("sendVoice", request)
    }

    /// Calls `sendVoice` using multipart upload for local bytes.
    /// `request.voice` is ignored; file content is taken from `file`.
    pub fn send_voice_upload(
        &self,
        request: &SendVoiceRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendVoice", request, "voice", file)
    }

    /// Calls `sendVideoNote`.
    pub fn send_video_note(&self, request: &SendVideoNoteRequest) -> Result<Message> {
        self.client.call_method("sendVideoNote", request)
    }

    /// Calls `sendVideoNote` using multipart upload for local bytes.
    /// `request.video_note` is ignored; file content is taken from `file`.
    pub fn send_video_note_upload(
        &self,
        request: &SendVideoNoteRequest,
        file: &UploadFile,
    ) -> Result<Message> {
        self.client
            .call_method_multipart("sendVideoNote", request, "video_note", file)
    }

    /// Calls `sendMediaGroup`.
    pub fn send_media_group(&self, request: &SendMediaGroupRequest) -> Result<Vec<Message>> {
        self.client.call_method("sendMediaGroup", request)
    }

    /// Calls `sendLocation`.
    pub fn send_location(&self, request: &SendLocationRequest) -> Result<Message> {
        self.client.call_method("sendLocation", request)
    }

    /// Calls `sendVenue`.
    pub fn send_venue(&self, request: &SendVenueRequest) -> Result<Message> {
        self.client.call_method("sendVenue", request)
    }

    /// Calls `sendContact`.
    pub fn send_contact(&self, request: &SendContactRequest) -> Result<Message> {
        self.client.call_method("sendContact", request)
    }

    /// Calls `sendPoll`.
    pub fn send_poll(&self, request: &SendPollRequest) -> Result<Message> {
        self.client.call_method("sendPoll", request)
    }

    /// Calls `stopPoll`.
    pub fn stop_poll(&self, request: &StopPollRequest) -> Result<Poll> {
        self.client.call_method("stopPoll", request)
    }

    /// Calls `sendDice`.
    pub fn send_dice(&self, request: &SendDiceRequest) -> Result<Message> {
        self.client.call_method("sendDice", request)
    }

    /// Calls `sendChatAction`.
    pub fn send_chat_action(&self, request: &SendChatActionRequest) -> Result<bool> {
        self.client.call_method("sendChatAction", request)
    }

    /// Calls `editMessageText`.
    pub fn edit_message_text(&self, request: &EditMessageTextRequest) -> Result<EditMessageResult> {
        request.validate()?;
        self.client.call_method("editMessageText", request)
    }

    /// Calls `editMessageCaption`.
    pub fn edit_message_caption(
        &self,
        request: &EditMessageCaptionRequest,
    ) -> Result<EditMessageResult> {
        request.validate()?;
        self.client.call_method("editMessageCaption", request)
    }

    /// Calls `editMessageReplyMarkup`.
    pub fn edit_message_reply_markup(
        &self,
        request: &EditMessageReplyMarkupRequest,
    ) -> Result<EditMessageResult> {
        request.validate()?;
        self.client.call_method("editMessageReplyMarkup", request)
    }

    /// Calls `editMessageLiveLocation`.
    pub fn edit_message_live_location(
        &self,
        request: &EditMessageLiveLocationRequest,
    ) -> Result<EditMessageResult> {
        request.validate()?;
        self.client.call_method("editMessageLiveLocation", request)
    }

    /// Calls `stopMessageLiveLocation`.
    pub fn stop_message_live_location(
        &self,
        request: &StopMessageLiveLocationRequest,
    ) -> Result<EditMessageResult> {
        request.validate()?;
        self.client.call_method("stopMessageLiveLocation", request)
    }

    /// Calls `deleteMessage`.
    pub fn delete_message(&self, request: &DeleteMessageRequest) -> Result<bool> {
        self.client.call_method("deleteMessage", request)
    }

    /// Calls `deleteMessages`.
    pub fn delete_messages(&self, request: &DeleteMessagesRequest) -> Result<bool> {
        self.client.call_method("deleteMessages", request)
    }
}
