use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::types::common::{ChatId, MessageId, UserId};

/// Typed request marker for advanced API methods.
pub trait AdvancedRequest: Serialize {
    type Response: DeserializeOwned;
    const METHOD: &'static str;
}

/// Auto-generated request for `addStickerToSet`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedAddStickerToSetRequest {
    pub user_id: UserId,
    pub name: String,
    pub sticker: crate::types::sticker::InputSticker,
}

impl AdvancedAddStickerToSetRequest {
    pub fn new(
        user_id: UserId,
        name: impl Into<String>,
        sticker: crate::types::sticker::InputSticker,
    ) -> Self {
        Self {
            user_id,
            name: name.into(),
            sticker,
        }
    }
}

impl AdvancedRequest for AdvancedAddStickerToSetRequest {
    type Response = bool;
    const METHOD: &'static str = "addStickerToSet";
}

/// Auto-generated request for `answerPreCheckoutQuery`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedAnswerPreCheckoutQueryRequest {
    pub pre_checkout_query_id: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl AdvancedAnswerPreCheckoutQueryRequest {
    pub fn new(pre_checkout_query_id: impl Into<String>, ok: bool) -> Self {
        Self {
            pre_checkout_query_id: pre_checkout_query_id.into(),
            ok,
            error_message: None,
        }
    }
}

impl AdvancedRequest for AdvancedAnswerPreCheckoutQueryRequest {
    type Response = bool;
    const METHOD: &'static str = "answerPreCheckoutQuery";
}

/// Auto-generated request for `answerShippingQuery`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedAnswerShippingQueryRequest {
    pub shipping_query_id: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shipping_options: Option<Vec<crate::types::payment::ShippingOption>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl AdvancedAnswerShippingQueryRequest {
    pub fn new(shipping_query_id: impl Into<String>, ok: bool) -> Self {
        Self {
            shipping_query_id: shipping_query_id.into(),
            ok,
            shipping_options: None,
            error_message: None,
        }
    }
}

impl AdvancedRequest for AdvancedAnswerShippingQueryRequest {
    type Response = bool;
    const METHOD: &'static str = "answerShippingQuery";
}

/// Auto-generated request for `answerWebAppQuery`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedAnswerWebAppQueryRequest {
    pub web_app_query_id: String,
    pub result: crate::types::telegram::InlineQueryResult,
}

impl AdvancedAnswerWebAppQueryRequest {
    pub fn new(
        web_app_query_id: impl Into<String>,
        result: crate::types::telegram::InlineQueryResult,
    ) -> Self {
        Self {
            web_app_query_id: web_app_query_id.into(),
            result,
        }
    }
}

impl AdvancedRequest for AdvancedAnswerWebAppQueryRequest {
    type Response = crate::types::message::SentWebAppMessage;
    const METHOD: &'static str = "answerWebAppQuery";
}

/// Auto-generated request for `approveChatJoinRequest`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedApproveChatJoinRequest {
    pub chat_id: ChatId,
    pub user_id: UserId,
}

impl AdvancedApproveChatJoinRequest {
    pub fn new(chat_id: impl Into<ChatId>, user_id: UserId) -> Self {
        Self {
            chat_id: chat_id.into(),
            user_id,
        }
    }
}

impl AdvancedRequest for AdvancedApproveChatJoinRequest {
    type Response = bool;
    const METHOD: &'static str = "approveChatJoinRequest";
}

/// Auto-generated request for `approveSuggestedPost`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedApproveSuggestedPostRequest {
    pub chat_id: i64,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_date: Option<i64>,
}

impl AdvancedApproveSuggestedPostRequest {
    pub fn new(chat_id: i64, message_id: MessageId) -> Self {
        Self {
            chat_id,
            message_id,
            send_date: None,
        }
    }
}

impl AdvancedRequest for AdvancedApproveSuggestedPostRequest {
    type Response = bool;
    const METHOD: &'static str = "approveSuggestedPost";
}

/// Auto-generated request for `closeForumTopic`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedCloseForumTopicRequest {
    pub chat_id: ChatId,
    pub message_thread_id: i64,
}

impl AdvancedCloseForumTopicRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_thread_id: i64) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_thread_id,
        }
    }
}

impl AdvancedRequest for AdvancedCloseForumTopicRequest {
    type Response = bool;
    const METHOD: &'static str = "closeForumTopic";
}

/// Auto-generated request for `closeGeneralForumTopic`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedCloseGeneralForumTopicRequest {
    pub chat_id: ChatId,
}

impl AdvancedCloseGeneralForumTopicRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
        }
    }
}

impl AdvancedRequest for AdvancedCloseGeneralForumTopicRequest {
    type Response = bool;
    const METHOD: &'static str = "closeGeneralForumTopic";
}

/// Auto-generated request for `convertGiftToStars`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedConvertGiftToStarsRequest {
    pub business_connection_id: String,
    pub owned_gift_id: String,
}

impl AdvancedConvertGiftToStarsRequest {
    pub fn new(
        business_connection_id: impl Into<String>,
        owned_gift_id: impl Into<String>,
    ) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            owned_gift_id: owned_gift_id.into(),
        }
    }
}

impl AdvancedRequest for AdvancedConvertGiftToStarsRequest {
    type Response = bool;
    const METHOD: &'static str = "convertGiftToStars";
}

/// Auto-generated request for `createChatSubscriptionInviteLink`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedCreateChatSubscriptionInviteLinkRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub subscription_period: i64,
    pub subscription_price: i64,
}

impl AdvancedCreateChatSubscriptionInviteLinkRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        subscription_period: i64,
        subscription_price: i64,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            name: None,
            subscription_period,
            subscription_price,
        }
    }
}

impl AdvancedRequest for AdvancedCreateChatSubscriptionInviteLinkRequest {
    type Response = crate::types::chat::ChatInviteLink;
    const METHOD: &'static str = "createChatSubscriptionInviteLink";
}

/// Auto-generated request for `createForumTopic`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedCreateForumTopicRequest {
    pub chat_id: ChatId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_color: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_custom_emoji_id: Option<String>,
}

impl AdvancedCreateForumTopicRequest {
    pub fn new(chat_id: impl Into<ChatId>, name: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            name: name.into(),
            icon_color: None,
            icon_custom_emoji_id: None,
        }
    }
}

impl AdvancedRequest for AdvancedCreateForumTopicRequest {
    type Response = Value;
    const METHOD: &'static str = "createForumTopic";
}

/// Auto-generated request for `createInvoiceLink`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedCreateInvoiceLinkRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    pub title: String,
    pub description: String,
    pub payload: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_token: Option<String>,
    pub currency: String,
    pub prices: Vec<crate::types::payment::LabeledPrice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_period: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tip_amount: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_tip_amounts: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_width: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_height: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_name: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_phone_number: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_email: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_shipping_address: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_phone_number_to_provider: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_email_to_provider: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_flexible: Option<bool>,
}

impl AdvancedCreateInvoiceLinkRequest {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        payload: impl Into<String>,
        currency: impl Into<String>,
        prices: Vec<crate::types::payment::LabeledPrice>,
    ) -> Self {
        Self {
            business_connection_id: None,
            title: title.into(),
            description: description.into(),
            payload: payload.into(),
            provider_token: None,
            currency: currency.into(),
            prices,
            subscription_period: None,
            max_tip_amount: None,
            suggested_tip_amounts: None,
            provider_data: None,
            photo_url: None,
            photo_size: None,
            photo_width: None,
            photo_height: None,
            need_name: None,
            need_phone_number: None,
            need_email: None,
            need_shipping_address: None,
            send_phone_number_to_provider: None,
            send_email_to_provider: None,
            is_flexible: None,
        }
    }
}

impl AdvancedRequest for AdvancedCreateInvoiceLinkRequest {
    type Response = String;
    const METHOD: &'static str = "createInvoiceLink";
}

/// Auto-generated request for `createNewStickerSet`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedCreateNewStickerSetRequest {
    pub user_id: UserId,
    pub name: String,
    pub title: String,
    pub stickers: Vec<crate::types::sticker::InputSticker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sticker_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs_repainting: Option<bool>,
}

impl AdvancedCreateNewStickerSetRequest {
    pub fn new(
        user_id: UserId,
        name: impl Into<String>,
        title: impl Into<String>,
        stickers: Vec<crate::types::sticker::InputSticker>,
    ) -> Self {
        Self {
            user_id,
            name: name.into(),
            title: title.into(),
            stickers,
            sticker_type: None,
            needs_repainting: None,
        }
    }
}

impl AdvancedRequest for AdvancedCreateNewStickerSetRequest {
    type Response = bool;
    const METHOD: &'static str = "createNewStickerSet";
}

/// Auto-generated request for `declineChatJoinRequest`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedDeclineChatJoinRequest {
    pub chat_id: ChatId,
    pub user_id: UserId,
}

impl AdvancedDeclineChatJoinRequest {
    pub fn new(chat_id: impl Into<ChatId>, user_id: UserId) -> Self {
        Self {
            chat_id: chat_id.into(),
            user_id,
        }
    }
}

impl AdvancedRequest for AdvancedDeclineChatJoinRequest {
    type Response = bool;
    const METHOD: &'static str = "declineChatJoinRequest";
}

/// Auto-generated request for `declineSuggestedPost`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedDeclineSuggestedPostRequest {
    pub chat_id: i64,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

impl AdvancedDeclineSuggestedPostRequest {
    pub fn new(chat_id: i64, message_id: MessageId) -> Self {
        Self {
            chat_id,
            message_id,
            comment: None,
        }
    }
}

impl AdvancedRequest for AdvancedDeclineSuggestedPostRequest {
    type Response = bool;
    const METHOD: &'static str = "declineSuggestedPost";
}

/// Auto-generated request for `deleteBusinessMessages`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedDeleteBusinessMessagesRequest {
    pub business_connection_id: String,
    pub message_ids: Vec<MessageId>,
}

impl AdvancedDeleteBusinessMessagesRequest {
    pub fn new(business_connection_id: impl Into<String>, message_ids: Vec<MessageId>) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            message_ids,
        }
    }
}

impl AdvancedRequest for AdvancedDeleteBusinessMessagesRequest {
    type Response = bool;
    const METHOD: &'static str = "deleteBusinessMessages";
}

/// Auto-generated request for `deleteForumTopic`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedDeleteForumTopicRequest {
    pub chat_id: ChatId,
    pub message_thread_id: i64,
}

impl AdvancedDeleteForumTopicRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_thread_id: i64) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_thread_id,
        }
    }
}

impl AdvancedRequest for AdvancedDeleteForumTopicRequest {
    type Response = bool;
    const METHOD: &'static str = "deleteForumTopic";
}

/// Auto-generated request for `deleteStickerFromSet`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedDeleteStickerFromSetRequest {
    pub sticker: String,
}

impl AdvancedDeleteStickerFromSetRequest {
    pub fn new(sticker: impl Into<String>) -> Self {
        Self {
            sticker: sticker.into(),
        }
    }
}

impl AdvancedRequest for AdvancedDeleteStickerFromSetRequest {
    type Response = bool;
    const METHOD: &'static str = "deleteStickerFromSet";
}

/// Auto-generated request for `deleteStickerSet`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedDeleteStickerSetRequest {
    pub name: String,
}

impl AdvancedDeleteStickerSetRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl AdvancedRequest for AdvancedDeleteStickerSetRequest {
    type Response = bool;
    const METHOD: &'static str = "deleteStickerSet";
}

/// Auto-generated request for `deleteStory`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedDeleteStoryRequest {
    pub business_connection_id: String,
    pub story_id: i64,
}

impl AdvancedDeleteStoryRequest {
    pub fn new(business_connection_id: impl Into<String>, story_id: i64) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            story_id,
        }
    }
}

impl AdvancedRequest for AdvancedDeleteStoryRequest {
    type Response = bool;
    const METHOD: &'static str = "deleteStory";
}

/// Auto-generated request for `editChatSubscriptionInviteLink`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedEditChatSubscriptionInviteLinkRequest {
    pub chat_id: ChatId,
    pub invite_link: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl AdvancedEditChatSubscriptionInviteLinkRequest {
    pub fn new(chat_id: impl Into<ChatId>, invite_link: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            invite_link: invite_link.into(),
            name: None,
        }
    }
}

impl AdvancedRequest for AdvancedEditChatSubscriptionInviteLinkRequest {
    type Response = crate::types::chat::ChatInviteLink;
    const METHOD: &'static str = "editChatSubscriptionInviteLink";
}

/// Auto-generated request for `editForumTopic`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedEditForumTopicRequest {
    pub chat_id: ChatId,
    pub message_thread_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_custom_emoji_id: Option<String>,
}

impl AdvancedEditForumTopicRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_thread_id: i64) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_thread_id,
            name: None,
            icon_custom_emoji_id: None,
        }
    }
}

impl AdvancedRequest for AdvancedEditForumTopicRequest {
    type Response = bool;
    const METHOD: &'static str = "editForumTopic";
}

/// Auto-generated request for `editGeneralForumTopic`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedEditGeneralForumTopicRequest {
    pub chat_id: ChatId,
    pub name: String,
}

impl AdvancedEditGeneralForumTopicRequest {
    pub fn new(chat_id: impl Into<ChatId>, name: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            name: name.into(),
        }
    }
}

impl AdvancedRequest for AdvancedEditGeneralForumTopicRequest {
    type Response = bool;
    const METHOD: &'static str = "editGeneralForumTopic";
}

/// Auto-generated request for `editMessageChecklist`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedEditMessageChecklistRequest {
    pub business_connection_id: String,
    pub chat_id: i64,
    pub message_id: MessageId,
    pub checklist: crate::types::telegram::InputChecklist,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<crate::types::telegram::InlineKeyboardMarkup>,
}

impl AdvancedEditMessageChecklistRequest {
    pub fn new(
        business_connection_id: impl Into<String>,
        chat_id: i64,
        message_id: MessageId,
        checklist: crate::types::telegram::InputChecklist,
    ) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            chat_id,
            message_id,
            checklist,
            reply_markup: None,
        }
    }
}

impl AdvancedRequest for AdvancedEditMessageChecklistRequest {
    type Response = crate::types::message::Message;
    const METHOD: &'static str = "editMessageChecklist";
}

/// Auto-generated request for `editMessageMedia`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedEditMessageMediaRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
    pub media: crate::types::message::InputMedia,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<crate::types::telegram::InlineKeyboardMarkup>,
}

impl AdvancedEditMessageMediaRequest {
    pub fn new(media: crate::types::message::InputMedia) -> Self {
        Self {
            business_connection_id: None,
            chat_id: None,
            message_id: None,
            inline_message_id: None,
            media,
            reply_markup: None,
        }
    }
}

impl AdvancedRequest for AdvancedEditMessageMediaRequest {
    type Response = crate::types::message::EditMessageResult;
    const METHOD: &'static str = "editMessageMedia";
}

/// Auto-generated request for `editStory`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedEditStoryRequest {
    pub business_connection_id: String,
    pub story_id: i64,
    pub content: crate::types::telegram::InputStoryContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<crate::types::message::MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub areas: Option<Vec<crate::types::telegram::StoryArea>>,
}

impl AdvancedEditStoryRequest {
    pub fn new(
        business_connection_id: impl Into<String>,
        story_id: i64,
        content: crate::types::telegram::InputStoryContent,
    ) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            story_id,
            content,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            areas: None,
        }
    }
}

impl AdvancedRequest for AdvancedEditStoryRequest {
    type Response = Value;
    const METHOD: &'static str = "editStory";
}

/// Auto-generated request for `editUserStarSubscription`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedEditUserStarSubscriptionRequest {
    pub user_id: UserId,
    pub telegram_payment_charge_id: String,
    pub is_canceled: bool,
}

impl AdvancedEditUserStarSubscriptionRequest {
    pub fn new(
        user_id: UserId,
        telegram_payment_charge_id: impl Into<String>,
        is_canceled: bool,
    ) -> Self {
        Self {
            user_id,
            telegram_payment_charge_id: telegram_payment_charge_id.into(),
            is_canceled,
        }
    }
}

impl AdvancedRequest for AdvancedEditUserStarSubscriptionRequest {
    type Response = bool;
    const METHOD: &'static str = "editUserStarSubscription";
}

/// Auto-generated request for `forwardMessages`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedForwardMessagesRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_messages_topic_id: Option<i64>,
    pub from_chat_id: ChatId,
    pub message_ids: Vec<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
}

impl AdvancedForwardMessagesRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        from_chat_id: impl Into<ChatId>,
        message_ids: Vec<MessageId>,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_thread_id: None,
            direct_messages_topic_id: None,
            from_chat_id: from_chat_id.into(),
            message_ids,
            disable_notification: None,
            protect_content: None,
        }
    }
}

impl AdvancedRequest for AdvancedForwardMessagesRequest {
    type Response = Vec<crate::types::message::MessageIdObject>;
    const METHOD: &'static str = "forwardMessages";
}

/// Auto-generated request for `getAvailableGifts`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AdvancedGetAvailableGiftsRequest {}

impl AdvancedGetAvailableGiftsRequest {
    pub fn new() -> Self {
        Self {}
    }
}

impl AdvancedRequest for AdvancedGetAvailableGiftsRequest {
    type Response = Value;
    const METHOD: &'static str = "getAvailableGifts";
}

/// Auto-generated request for `getBusinessAccountGifts`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGetBusinessAccountGiftsRequest {
    pub business_connection_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_unsaved: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_saved: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_unlimited: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_limited_upgradable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_limited_non_upgradable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_unique: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_from_blockchain: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_by_price: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl AdvancedGetBusinessAccountGiftsRequest {
    pub fn new(business_connection_id: impl Into<String>) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            exclude_unsaved: None,
            exclude_saved: None,
            exclude_unlimited: None,
            exclude_limited_upgradable: None,
            exclude_limited_non_upgradable: None,
            exclude_unique: None,
            exclude_from_blockchain: None,
            sort_by_price: None,
            offset: None,
            limit: None,
        }
    }
}

impl AdvancedRequest for AdvancedGetBusinessAccountGiftsRequest {
    type Response = Value;
    const METHOD: &'static str = "getBusinessAccountGifts";
}

/// Auto-generated request for `getBusinessAccountStarBalance`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGetBusinessAccountStarBalanceRequest {
    pub business_connection_id: String,
}

impl AdvancedGetBusinessAccountStarBalanceRequest {
    pub fn new(business_connection_id: impl Into<String>) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
        }
    }
}

impl AdvancedRequest for AdvancedGetBusinessAccountStarBalanceRequest {
    type Response = Value;
    const METHOD: &'static str = "getBusinessAccountStarBalance";
}

/// Auto-generated request for `getBusinessConnection`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGetBusinessConnectionRequest {
    pub business_connection_id: String,
}

impl AdvancedGetBusinessConnectionRequest {
    pub fn new(business_connection_id: impl Into<String>) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
        }
    }
}

impl AdvancedRequest for AdvancedGetBusinessConnectionRequest {
    type Response = Value;
    const METHOD: &'static str = "getBusinessConnection";
}

/// Auto-generated request for `getChatGifts`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGetChatGiftsRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_unsaved: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_saved: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_unlimited: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_limited_upgradable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_limited_non_upgradable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_from_blockchain: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_unique: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_by_price: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl AdvancedGetChatGiftsRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            exclude_unsaved: None,
            exclude_saved: None,
            exclude_unlimited: None,
            exclude_limited_upgradable: None,
            exclude_limited_non_upgradable: None,
            exclude_from_blockchain: None,
            exclude_unique: None,
            sort_by_price: None,
            offset: None,
            limit: None,
        }
    }
}

impl AdvancedRequest for AdvancedGetChatGiftsRequest {
    type Response = Value;
    const METHOD: &'static str = "getChatGifts";
}

/// Auto-generated request for `getChatMenuButton`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AdvancedGetChatMenuButtonRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<i64>,
}

impl AdvancedGetChatMenuButtonRequest {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AdvancedRequest for AdvancedGetChatMenuButtonRequest {
    type Response = crate::types::telegram::MenuButton;
    const METHOD: &'static str = "getChatMenuButton";
}

/// Auto-generated request for `getCustomEmojiStickers`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGetCustomEmojiStickersRequest {
    pub custom_emoji_ids: Vec<String>,
}

impl AdvancedGetCustomEmojiStickersRequest {
    pub fn new(custom_emoji_ids: Vec<String>) -> Self {
        Self { custom_emoji_ids }
    }
}

impl AdvancedRequest for AdvancedGetCustomEmojiStickersRequest {
    type Response = Vec<crate::types::sticker::Sticker>;
    const METHOD: &'static str = "getCustomEmojiStickers";
}

/// Auto-generated request for `getForumTopicIconStickers`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AdvancedGetForumTopicIconStickersRequest {}

impl AdvancedGetForumTopicIconStickersRequest {
    pub fn new() -> Self {
        Self {}
    }
}

impl AdvancedRequest for AdvancedGetForumTopicIconStickersRequest {
    type Response = Vec<crate::types::sticker::Sticker>;
    const METHOD: &'static str = "getForumTopicIconStickers";
}

/// Auto-generated request for `getGameHighScores`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGetGameHighScoresRequest {
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
}

impl AdvancedGetGameHighScoresRequest {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            chat_id: None,
            message_id: None,
            inline_message_id: None,
        }
    }
}

impl AdvancedRequest for AdvancedGetGameHighScoresRequest {
    type Response = Value;
    const METHOD: &'static str = "getGameHighScores";
}

/// Auto-generated request for `getMyDefaultAdministratorRights`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AdvancedGetMyDefaultAdministratorRightsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub for_channels: Option<bool>,
}

impl AdvancedGetMyDefaultAdministratorRightsRequest {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AdvancedRequest for AdvancedGetMyDefaultAdministratorRightsRequest {
    type Response = crate::types::chat::ChatAdministratorRights;
    const METHOD: &'static str = "getMyDefaultAdministratorRights";
}

/// Auto-generated request for `getMyStarBalance`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AdvancedGetMyStarBalanceRequest {}

impl AdvancedGetMyStarBalanceRequest {
    pub fn new() -> Self {
        Self {}
    }
}

impl AdvancedRequest for AdvancedGetMyStarBalanceRequest {
    type Response = Value;
    const METHOD: &'static str = "getMyStarBalance";
}

/// Auto-generated request for `getStarTransactions`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AdvancedGetStarTransactionsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl AdvancedGetStarTransactionsRequest {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AdvancedRequest for AdvancedGetStarTransactionsRequest {
    type Response = Value;
    const METHOD: &'static str = "getStarTransactions";
}

/// Auto-generated request for `getStickerSet`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGetStickerSetRequest {
    pub name: String,
}

impl AdvancedGetStickerSetRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl AdvancedRequest for AdvancedGetStickerSetRequest {
    type Response = crate::types::sticker::StickerSet;
    const METHOD: &'static str = "getStickerSet";
}

/// Auto-generated request for `getUserChatBoosts`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGetUserChatBoostsRequest {
    pub chat_id: ChatId,
    pub user_id: UserId,
}

impl AdvancedGetUserChatBoostsRequest {
    pub fn new(chat_id: impl Into<ChatId>, user_id: UserId) -> Self {
        Self {
            chat_id: chat_id.into(),
            user_id,
        }
    }
}

impl AdvancedRequest for AdvancedGetUserChatBoostsRequest {
    type Response = Value;
    const METHOD: &'static str = "getUserChatBoosts";
}

/// Auto-generated request for `getUserGifts`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGetUserGiftsRequest {
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_unlimited: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_limited_upgradable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_limited_non_upgradable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_from_blockchain: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude_unique: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_by_price: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl AdvancedGetUserGiftsRequest {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            exclude_unlimited: None,
            exclude_limited_upgradable: None,
            exclude_limited_non_upgradable: None,
            exclude_from_blockchain: None,
            exclude_unique: None,
            sort_by_price: None,
            offset: None,
            limit: None,
        }
    }
}

impl AdvancedRequest for AdvancedGetUserGiftsRequest {
    type Response = Value;
    const METHOD: &'static str = "getUserGifts";
}

/// Auto-generated request for `getUserProfileAudios`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGetUserProfileAudiosRequest {
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl AdvancedGetUserProfileAudiosRequest {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            offset: None,
            limit: None,
        }
    }
}

impl AdvancedRequest for AdvancedGetUserProfileAudiosRequest {
    type Response = Value;
    const METHOD: &'static str = "getUserProfileAudios";
}

/// Auto-generated request for `giftPremiumSubscription`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedGiftPremiumSubscriptionRequest {
    pub user_id: UserId,
    pub month_count: i64,
    pub star_count: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_parse_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<crate::types::message::MessageEntity>>,
}

impl AdvancedGiftPremiumSubscriptionRequest {
    pub fn new(user_id: UserId, month_count: i64, star_count: i64) -> Self {
        Self {
            user_id,
            month_count,
            star_count,
            text: None,
            text_parse_mode: None,
            text_entities: None,
        }
    }
}

impl AdvancedRequest for AdvancedGiftPremiumSubscriptionRequest {
    type Response = bool;
    const METHOD: &'static str = "giftPremiumSubscription";
}

/// Auto-generated request for `hideGeneralForumTopic`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedHideGeneralForumTopicRequest {
    pub chat_id: ChatId,
}

impl AdvancedHideGeneralForumTopicRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
        }
    }
}

impl AdvancedRequest for AdvancedHideGeneralForumTopicRequest {
    type Response = bool;
    const METHOD: &'static str = "hideGeneralForumTopic";
}

/// Auto-generated request for `postStory`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedPostStoryRequest {
    pub business_connection_id: String,
    pub content: crate::types::telegram::InputStoryContent,
    pub active_period: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<crate::types::message::MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub areas: Option<Vec<crate::types::telegram::StoryArea>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_to_chat_page: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
}

impl AdvancedPostStoryRequest {
    pub fn new(
        business_connection_id: impl Into<String>,
        content: crate::types::telegram::InputStoryContent,
        active_period: i64,
    ) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            content,
            active_period,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            areas: None,
            post_to_chat_page: None,
            protect_content: None,
        }
    }
}

impl AdvancedRequest for AdvancedPostStoryRequest {
    type Response = Value;
    const METHOD: &'static str = "postStory";
}

/// Auto-generated request for `readBusinessMessage`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedReadBusinessMessageRequest {
    pub business_connection_id: String,
    pub chat_id: i64,
    pub message_id: MessageId,
}

impl AdvancedReadBusinessMessageRequest {
    pub fn new(
        business_connection_id: impl Into<String>,
        chat_id: i64,
        message_id: MessageId,
    ) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            chat_id,
            message_id,
        }
    }
}

impl AdvancedRequest for AdvancedReadBusinessMessageRequest {
    type Response = bool;
    const METHOD: &'static str = "readBusinessMessage";
}

/// Auto-generated request for `refundStarPayment`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedRefundStarPaymentRequest {
    pub user_id: UserId,
    pub telegram_payment_charge_id: String,
}

impl AdvancedRefundStarPaymentRequest {
    pub fn new(user_id: UserId, telegram_payment_charge_id: impl Into<String>) -> Self {
        Self {
            user_id,
            telegram_payment_charge_id: telegram_payment_charge_id.into(),
        }
    }
}

impl AdvancedRequest for AdvancedRefundStarPaymentRequest {
    type Response = bool;
    const METHOD: &'static str = "refundStarPayment";
}

/// Auto-generated request for `removeBusinessAccountProfilePhoto`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedRemoveBusinessAccountProfilePhotoRequest {
    pub business_connection_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

impl AdvancedRemoveBusinessAccountProfilePhotoRequest {
    pub fn new(business_connection_id: impl Into<String>) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            is_public: None,
        }
    }
}

impl AdvancedRequest for AdvancedRemoveBusinessAccountProfilePhotoRequest {
    type Response = bool;
    const METHOD: &'static str = "removeBusinessAccountProfilePhoto";
}

/// Auto-generated request for `removeChatVerification`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedRemoveChatVerificationRequest {
    pub chat_id: ChatId,
}

impl AdvancedRemoveChatVerificationRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
        }
    }
}

impl AdvancedRequest for AdvancedRemoveChatVerificationRequest {
    type Response = bool;
    const METHOD: &'static str = "removeChatVerification";
}

/// Auto-generated request for `removeMyProfilePhoto`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AdvancedRemoveMyProfilePhotoRequest {}

impl AdvancedRemoveMyProfilePhotoRequest {
    pub fn new() -> Self {
        Self {}
    }
}

impl AdvancedRequest for AdvancedRemoveMyProfilePhotoRequest {
    type Response = bool;
    const METHOD: &'static str = "removeMyProfilePhoto";
}

/// Auto-generated request for `removeUserVerification`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedRemoveUserVerificationRequest {
    pub user_id: UserId,
}

impl AdvancedRemoveUserVerificationRequest {
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}

impl AdvancedRequest for AdvancedRemoveUserVerificationRequest {
    type Response = bool;
    const METHOD: &'static str = "removeUserVerification";
}

/// Auto-generated request for `reopenForumTopic`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedReopenForumTopicRequest {
    pub chat_id: ChatId,
    pub message_thread_id: i64,
}

impl AdvancedReopenForumTopicRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_thread_id: i64) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_thread_id,
        }
    }
}

impl AdvancedRequest for AdvancedReopenForumTopicRequest {
    type Response = bool;
    const METHOD: &'static str = "reopenForumTopic";
}

/// Auto-generated request for `reopenGeneralForumTopic`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedReopenGeneralForumTopicRequest {
    pub chat_id: ChatId,
}

impl AdvancedReopenGeneralForumTopicRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
        }
    }
}

impl AdvancedRequest for AdvancedReopenGeneralForumTopicRequest {
    type Response = bool;
    const METHOD: &'static str = "reopenGeneralForumTopic";
}

/// Auto-generated request for `replaceStickerInSet`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedReplaceStickerInSetRequest {
    pub user_id: UserId,
    pub name: String,
    pub old_sticker: String,
    pub sticker: crate::types::sticker::InputSticker,
}

impl AdvancedReplaceStickerInSetRequest {
    pub fn new(
        user_id: UserId,
        name: impl Into<String>,
        old_sticker: impl Into<String>,
        sticker: crate::types::sticker::InputSticker,
    ) -> Self {
        Self {
            user_id,
            name: name.into(),
            old_sticker: old_sticker.into(),
            sticker,
        }
    }
}

impl AdvancedRequest for AdvancedReplaceStickerInSetRequest {
    type Response = bool;
    const METHOD: &'static str = "replaceStickerInSet";
}

/// Auto-generated request for `repostStory`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedRepostStoryRequest {
    pub business_connection_id: String,
    pub from_chat_id: i64,
    pub from_story_id: i64,
    pub active_period: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post_to_chat_page: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
}

impl AdvancedRepostStoryRequest {
    pub fn new(
        business_connection_id: impl Into<String>,
        from_chat_id: i64,
        from_story_id: i64,
        active_period: i64,
    ) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            from_chat_id,
            from_story_id,
            active_period,
            post_to_chat_page: None,
            protect_content: None,
        }
    }
}

impl AdvancedRequest for AdvancedRepostStoryRequest {
    type Response = Value;
    const METHOD: &'static str = "repostStory";
}

/// Auto-generated request for `savePreparedInlineMessage`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSavePreparedInlineMessageRequest {
    pub user_id: UserId,
    pub result: crate::types::telegram::InlineQueryResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_user_chats: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_bot_chats: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_group_chats: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_channel_chats: Option<bool>,
}

impl AdvancedSavePreparedInlineMessageRequest {
    pub fn new(user_id: UserId, result: crate::types::telegram::InlineQueryResult) -> Self {
        Self {
            user_id,
            result,
            allow_user_chats: None,
            allow_bot_chats: None,
            allow_group_chats: None,
            allow_channel_chats: None,
        }
    }
}

impl AdvancedRequest for AdvancedSavePreparedInlineMessageRequest {
    type Response = crate::types::message::Message;
    const METHOD: &'static str = "savePreparedInlineMessage";
}

/// Auto-generated request for `sendChecklist`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSendChecklistRequest {
    pub business_connection_id: String,
    pub chat_id: i64,
    pub checklist: crate::types::telegram::InputChecklist,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<crate::types::telegram::ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<crate::types::telegram::InlineKeyboardMarkup>,
}

impl AdvancedSendChecklistRequest {
    pub fn new(
        business_connection_id: impl Into<String>,
        chat_id: i64,
        checklist: crate::types::telegram::InputChecklist,
    ) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            chat_id,
            checklist,
            disable_notification: None,
            protect_content: None,
            message_effect_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

impl AdvancedRequest for AdvancedSendChecklistRequest {
    type Response = crate::types::message::Message;
    const METHOD: &'static str = "sendChecklist";
}

/// Auto-generated request for `sendGame`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSendGameRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    pub chat_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    pub game_short_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_paid_broadcast: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<crate::types::telegram::ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<crate::types::telegram::InlineKeyboardMarkup>,
}

impl AdvancedSendGameRequest {
    pub fn new(chat_id: i64, game_short_name: impl Into<String>) -> Self {
        Self {
            business_connection_id: None,
            chat_id,
            message_thread_id: None,
            game_short_name: game_short_name.into(),
            disable_notification: None,
            protect_content: None,
            allow_paid_broadcast: None,
            message_effect_id: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

impl AdvancedRequest for AdvancedSendGameRequest {
    type Response = crate::types::message::Message;
    const METHOD: &'static str = "sendGame";
}

/// Auto-generated request for `sendGift`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSendGiftRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<ChatId>,
    pub gift_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pay_for_upgrade: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_parse_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_entities: Option<Vec<crate::types::message::MessageEntity>>,
}

impl AdvancedSendGiftRequest {
    pub fn new(gift_id: impl Into<String>) -> Self {
        Self {
            user_id: None,
            chat_id: None,
            gift_id: gift_id.into(),
            pay_for_upgrade: None,
            text: None,
            text_parse_mode: None,
            text_entities: None,
        }
    }
}

impl AdvancedRequest for AdvancedSendGiftRequest {
    type Response = bool;
    const METHOD: &'static str = "sendGift";
}

/// Auto-generated request for `sendInvoice`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSendInvoiceRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_messages_topic_id: Option<i64>,
    pub title: String,
    pub description: String,
    pub payload: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_token: Option<String>,
    pub currency: String,
    pub prices: Vec<crate::types::payment::LabeledPrice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tip_amount: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_tip_amounts: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_parameter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_width: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_height: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_name: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_phone_number: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_email: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need_shipping_address: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_phone_number_to_provider: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub send_email_to_provider: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_flexible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_paid_broadcast: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_parameters: Option<crate::types::telegram::SuggestedPostParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<crate::types::telegram::ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<crate::types::telegram::InlineKeyboardMarkup>,
}

impl AdvancedSendInvoiceRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        title: impl Into<String>,
        description: impl Into<String>,
        payload: impl Into<String>,
        currency: impl Into<String>,
        prices: Vec<crate::types::payment::LabeledPrice>,
    ) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_thread_id: None,
            direct_messages_topic_id: None,
            title: title.into(),
            description: description.into(),
            payload: payload.into(),
            provider_token: None,
            currency: currency.into(),
            prices,
            max_tip_amount: None,
            suggested_tip_amounts: None,
            start_parameter: None,
            provider_data: None,
            photo_url: None,
            photo_size: None,
            photo_width: None,
            photo_height: None,
            need_name: None,
            need_phone_number: None,
            need_email: None,
            need_shipping_address: None,
            send_phone_number_to_provider: None,
            send_email_to_provider: None,
            is_flexible: None,
            disable_notification: None,
            protect_content: None,
            allow_paid_broadcast: None,
            message_effect_id: None,
            suggested_post_parameters: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

impl AdvancedRequest for AdvancedSendInvoiceRequest {
    type Response = crate::types::message::Message;
    const METHOD: &'static str = "sendInvoice";
}

/// Auto-generated request for `sendMessageDraft`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSendMessageDraftRequest {
    pub chat_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    pub draft_id: i64,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entities: Option<Vec<crate::types::message::MessageEntity>>,
}

impl AdvancedSendMessageDraftRequest {
    pub fn new(chat_id: i64, draft_id: i64, text: impl Into<String>) -> Self {
        Self {
            chat_id,
            message_thread_id: None,
            draft_id,
            text: text.into(),
            parse_mode: None,
            entities: None,
        }
    }
}

impl AdvancedRequest for AdvancedSendMessageDraftRequest {
    type Response = bool;
    const METHOD: &'static str = "sendMessageDraft";
}

/// Auto-generated request for `sendPaidMedia`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSendPaidMediaRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_messages_topic_id: Option<i64>,
    pub star_count: i64,
    pub media: Vec<crate::types::telegram::InputPaidMedia>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub caption_entities: Option<Vec<crate::types::message::MessageEntity>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_caption_above_media: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_paid_broadcast: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_parameters: Option<crate::types::telegram::SuggestedPostParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<crate::types::telegram::ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<crate::types::telegram::ReplyMarkup>,
}

impl AdvancedSendPaidMediaRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        star_count: i64,
        media: Vec<crate::types::telegram::InputPaidMedia>,
    ) -> Self {
        Self {
            business_connection_id: None,
            chat_id: chat_id.into(),
            message_thread_id: None,
            direct_messages_topic_id: None,
            star_count,
            media,
            payload: None,
            caption: None,
            parse_mode: None,
            caption_entities: None,
            show_caption_above_media: None,
            disable_notification: None,
            protect_content: None,
            allow_paid_broadcast: None,
            suggested_post_parameters: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

impl AdvancedRequest for AdvancedSendPaidMediaRequest {
    type Response = crate::types::message::Message;
    const METHOD: &'static str = "sendPaidMedia";
}

/// Auto-generated request for `sendSticker`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSendStickerRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_messages_topic_id: Option<i64>,
    pub sticker: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_notification: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protect_content: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_paid_broadcast: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_post_parameters: Option<crate::types::telegram::SuggestedPostParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<crate::types::telegram::ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<crate::types::telegram::ReplyMarkup>,
}

impl AdvancedSendStickerRequest {
    pub fn new(chat_id: impl Into<ChatId>, sticker: impl Into<String>) -> Self {
        Self {
            business_connection_id: None,
            chat_id: chat_id.into(),
            message_thread_id: None,
            direct_messages_topic_id: None,
            sticker: sticker.into(),
            emoji: None,
            disable_notification: None,
            protect_content: None,
            allow_paid_broadcast: None,
            message_effect_id: None,
            suggested_post_parameters: None,
            reply_parameters: None,
            reply_markup: None,
        }
    }
}

impl AdvancedRequest for AdvancedSendStickerRequest {
    type Response = crate::types::message::Message;
    const METHOD: &'static str = "sendSticker";
}

/// Auto-generated request for `setBusinessAccountBio`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetBusinessAccountBioRequest {
    pub business_connection_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
}

impl AdvancedSetBusinessAccountBioRequest {
    pub fn new(business_connection_id: impl Into<String>) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            bio: None,
        }
    }
}

impl AdvancedRequest for AdvancedSetBusinessAccountBioRequest {
    type Response = bool;
    const METHOD: &'static str = "setBusinessAccountBio";
}

/// Auto-generated request for `setBusinessAccountGiftSettings`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetBusinessAccountGiftSettingsRequest {
    pub business_connection_id: String,
    pub show_gift_button: bool,
    pub accepted_gift_types: crate::types::telegram::AcceptedGiftTypes,
}

impl AdvancedSetBusinessAccountGiftSettingsRequest {
    pub fn new(
        business_connection_id: impl Into<String>,
        show_gift_button: bool,
        accepted_gift_types: crate::types::telegram::AcceptedGiftTypes,
    ) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            show_gift_button,
            accepted_gift_types,
        }
    }
}

impl AdvancedRequest for AdvancedSetBusinessAccountGiftSettingsRequest {
    type Response = bool;
    const METHOD: &'static str = "setBusinessAccountGiftSettings";
}

/// Auto-generated request for `setBusinessAccountName`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetBusinessAccountNameRequest {
    pub business_connection_id: String,
    pub first_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
}

impl AdvancedSetBusinessAccountNameRequest {
    pub fn new(business_connection_id: impl Into<String>, first_name: impl Into<String>) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            first_name: first_name.into(),
            last_name: None,
        }
    }
}

impl AdvancedRequest for AdvancedSetBusinessAccountNameRequest {
    type Response = bool;
    const METHOD: &'static str = "setBusinessAccountName";
}

/// Auto-generated request for `setBusinessAccountProfilePhoto`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetBusinessAccountProfilePhotoRequest {
    pub business_connection_id: String,
    pub photo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

impl AdvancedSetBusinessAccountProfilePhotoRequest {
    pub fn new(business_connection_id: impl Into<String>, photo: impl Into<String>) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            photo: photo.into(),
            is_public: None,
        }
    }
}

impl AdvancedRequest for AdvancedSetBusinessAccountProfilePhotoRequest {
    type Response = bool;
    const METHOD: &'static str = "setBusinessAccountProfilePhoto";
}

/// Auto-generated request for `setBusinessAccountUsername`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetBusinessAccountUsernameRequest {
    pub business_connection_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

impl AdvancedSetBusinessAccountUsernameRequest {
    pub fn new(business_connection_id: impl Into<String>) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            username: None,
        }
    }
}

impl AdvancedRequest for AdvancedSetBusinessAccountUsernameRequest {
    type Response = bool;
    const METHOD: &'static str = "setBusinessAccountUsername";
}

/// Auto-generated request for `setChatMenuButton`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AdvancedSetChatMenuButtonRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub menu_button: Option<crate::types::telegram::MenuButton>,
}

impl AdvancedSetChatMenuButtonRequest {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AdvancedRequest for AdvancedSetChatMenuButtonRequest {
    type Response = bool;
    const METHOD: &'static str = "setChatMenuButton";
}

/// Auto-generated request for `setChatPhoto`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetChatPhotoRequest {
    pub chat_id: ChatId,
    pub photo: String,
}

impl AdvancedSetChatPhotoRequest {
    pub fn new(chat_id: impl Into<ChatId>, photo: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            photo: photo.into(),
        }
    }
}

impl AdvancedRequest for AdvancedSetChatPhotoRequest {
    type Response = bool;
    const METHOD: &'static str = "setChatPhoto";
}

/// Auto-generated request for `setCustomEmojiStickerSetThumbnail`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetCustomEmojiStickerSetThumbnailRequest {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_emoji_id: Option<String>,
}

impl AdvancedSetCustomEmojiStickerSetThumbnailRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            custom_emoji_id: None,
        }
    }
}

impl AdvancedRequest for AdvancedSetCustomEmojiStickerSetThumbnailRequest {
    type Response = bool;
    const METHOD: &'static str = "setCustomEmojiStickerSetThumbnail";
}

/// Auto-generated request for `setGameScore`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetGameScoreRequest {
    pub user_id: UserId,
    pub score: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_edit_message: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inline_message_id: Option<String>,
}

impl AdvancedSetGameScoreRequest {
    pub fn new(user_id: UserId, score: i64) -> Self {
        Self {
            user_id,
            score,
            force: None,
            disable_edit_message: None,
            chat_id: None,
            message_id: None,
            inline_message_id: None,
        }
    }
}

impl AdvancedRequest for AdvancedSetGameScoreRequest {
    type Response = Value;
    const METHOD: &'static str = "setGameScore";
}

/// Auto-generated request for `setMessageReaction`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetMessageReactionRequest {
    pub chat_id: ChatId,
    pub message_id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reaction: Option<Vec<crate::types::telegram::ReactionType>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_big: Option<bool>,
}

impl AdvancedSetMessageReactionRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_id: MessageId) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_id,
            reaction: None,
            is_big: None,
        }
    }
}

impl AdvancedRequest for AdvancedSetMessageReactionRequest {
    type Response = bool;
    const METHOD: &'static str = "setMessageReaction";
}

/// Auto-generated request for `setMyDefaultAdministratorRights`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AdvancedSetMyDefaultAdministratorRightsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rights: Option<crate::types::chat::ChatAdministratorRights>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub for_channels: Option<bool>,
}

impl AdvancedSetMyDefaultAdministratorRightsRequest {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AdvancedRequest for AdvancedSetMyDefaultAdministratorRightsRequest {
    type Response = bool;
    const METHOD: &'static str = "setMyDefaultAdministratorRights";
}

/// Auto-generated request for `setMyProfilePhoto`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetMyProfilePhotoRequest {
    pub photo: String,
}

impl AdvancedSetMyProfilePhotoRequest {
    pub fn new(photo: impl Into<String>) -> Self {
        Self {
            photo: photo.into(),
        }
    }
}

impl AdvancedRequest for AdvancedSetMyProfilePhotoRequest {
    type Response = bool;
    const METHOD: &'static str = "setMyProfilePhoto";
}

/// Auto-generated request for `setPassportDataErrors`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetPassportDataErrorsRequest {
    pub user_id: UserId,
    pub errors: Vec<crate::types::telegram::PassportElementError>,
}

impl AdvancedSetPassportDataErrorsRequest {
    pub fn new(user_id: UserId, errors: Vec<crate::types::telegram::PassportElementError>) -> Self {
        Self { user_id, errors }
    }
}

impl AdvancedRequest for AdvancedSetPassportDataErrorsRequest {
    type Response = bool;
    const METHOD: &'static str = "setPassportDataErrors";
}

/// Auto-generated request for `setStickerEmojiList`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetStickerEmojiListRequest {
    pub sticker: String,
    pub emoji_list: Vec<String>,
}

impl AdvancedSetStickerEmojiListRequest {
    pub fn new(sticker: impl Into<String>, emoji_list: Vec<String>) -> Self {
        Self {
            sticker: sticker.into(),
            emoji_list,
        }
    }
}

impl AdvancedRequest for AdvancedSetStickerEmojiListRequest {
    type Response = bool;
    const METHOD: &'static str = "setStickerEmojiList";
}

/// Auto-generated request for `setStickerKeywords`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetStickerKeywordsRequest {
    pub sticker: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
}

impl AdvancedSetStickerKeywordsRequest {
    pub fn new(sticker: impl Into<String>) -> Self {
        Self {
            sticker: sticker.into(),
            keywords: None,
        }
    }
}

impl AdvancedRequest for AdvancedSetStickerKeywordsRequest {
    type Response = bool;
    const METHOD: &'static str = "setStickerKeywords";
}

/// Auto-generated request for `setStickerMaskPosition`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetStickerMaskPositionRequest {
    pub sticker: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mask_position: Option<crate::types::sticker::MaskPosition>,
}

impl AdvancedSetStickerMaskPositionRequest {
    pub fn new(sticker: impl Into<String>) -> Self {
        Self {
            sticker: sticker.into(),
            mask_position: None,
        }
    }
}

impl AdvancedRequest for AdvancedSetStickerMaskPositionRequest {
    type Response = bool;
    const METHOD: &'static str = "setStickerMaskPosition";
}

/// Auto-generated request for `setStickerPositionInSet`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetStickerPositionInSetRequest {
    pub sticker: String,
    pub position: i64,
}

impl AdvancedSetStickerPositionInSetRequest {
    pub fn new(sticker: impl Into<String>, position: i64) -> Self {
        Self {
            sticker: sticker.into(),
            position,
        }
    }
}

impl AdvancedRequest for AdvancedSetStickerPositionInSetRequest {
    type Response = bool;
    const METHOD: &'static str = "setStickerPositionInSet";
}

/// Auto-generated request for `setStickerSetThumbnail`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetStickerSetThumbnailRequest {
    pub name: String,
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    pub format: String,
}

impl AdvancedSetStickerSetThumbnailRequest {
    pub fn new(name: impl Into<String>, user_id: UserId, format: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            user_id,
            thumbnail: None,
            format: format.into(),
        }
    }
}

impl AdvancedRequest for AdvancedSetStickerSetThumbnailRequest {
    type Response = bool;
    const METHOD: &'static str = "setStickerSetThumbnail";
}

/// Auto-generated request for `setStickerSetTitle`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetStickerSetTitleRequest {
    pub name: String,
    pub title: String,
}

impl AdvancedSetStickerSetTitleRequest {
    pub fn new(name: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            title: title.into(),
        }
    }
}

impl AdvancedRequest for AdvancedSetStickerSetTitleRequest {
    type Response = bool;
    const METHOD: &'static str = "setStickerSetTitle";
}

/// Auto-generated request for `setUserEmojiStatus`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedSetUserEmojiStatusRequest {
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji_status_custom_emoji_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emoji_status_expiration_date: Option<i64>,
}

impl AdvancedSetUserEmojiStatusRequest {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            emoji_status_custom_emoji_id: None,
            emoji_status_expiration_date: None,
        }
    }
}

impl AdvancedRequest for AdvancedSetUserEmojiStatusRequest {
    type Response = bool;
    const METHOD: &'static str = "setUserEmojiStatus";
}

/// Auto-generated request for `transferBusinessAccountStars`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedTransferBusinessAccountStarsRequest {
    pub business_connection_id: String,
    pub star_count: i64,
}

impl AdvancedTransferBusinessAccountStarsRequest {
    pub fn new(business_connection_id: impl Into<String>, star_count: i64) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            star_count,
        }
    }
}

impl AdvancedRequest for AdvancedTransferBusinessAccountStarsRequest {
    type Response = bool;
    const METHOD: &'static str = "transferBusinessAccountStars";
}

/// Auto-generated request for `transferGift`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedTransferGiftRequest {
    pub business_connection_id: String,
    pub owned_gift_id: String,
    pub new_owner_chat_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub star_count: Option<i64>,
}

impl AdvancedTransferGiftRequest {
    pub fn new(
        business_connection_id: impl Into<String>,
        owned_gift_id: impl Into<String>,
        new_owner_chat_id: i64,
    ) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            owned_gift_id: owned_gift_id.into(),
            new_owner_chat_id,
            star_count: None,
        }
    }
}

impl AdvancedRequest for AdvancedTransferGiftRequest {
    type Response = bool;
    const METHOD: &'static str = "transferGift";
}

/// Auto-generated request for `unhideGeneralForumTopic`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedUnhideGeneralForumTopicRequest {
    pub chat_id: ChatId,
}

impl AdvancedUnhideGeneralForumTopicRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
        }
    }
}

impl AdvancedRequest for AdvancedUnhideGeneralForumTopicRequest {
    type Response = bool;
    const METHOD: &'static str = "unhideGeneralForumTopic";
}

/// Auto-generated request for `unpinAllForumTopicMessages`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedUnpinAllForumTopicMessagesRequest {
    pub chat_id: ChatId,
    pub message_thread_id: i64,
}

impl AdvancedUnpinAllForumTopicMessagesRequest {
    pub fn new(chat_id: impl Into<ChatId>, message_thread_id: i64) -> Self {
        Self {
            chat_id: chat_id.into(),
            message_thread_id,
        }
    }
}

impl AdvancedRequest for AdvancedUnpinAllForumTopicMessagesRequest {
    type Response = bool;
    const METHOD: &'static str = "unpinAllForumTopicMessages";
}

/// Auto-generated request for `unpinAllGeneralForumTopicMessages`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedUnpinAllGeneralForumTopicMessagesRequest {
    pub chat_id: ChatId,
}

impl AdvancedUnpinAllGeneralForumTopicMessagesRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
        }
    }
}

impl AdvancedRequest for AdvancedUnpinAllGeneralForumTopicMessagesRequest {
    type Response = bool;
    const METHOD: &'static str = "unpinAllGeneralForumTopicMessages";
}

/// Auto-generated request for `upgradeGift`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedUpgradeGiftRequest {
    pub business_connection_id: String,
    pub owned_gift_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keep_original_details: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub star_count: Option<i64>,
}

impl AdvancedUpgradeGiftRequest {
    pub fn new(
        business_connection_id: impl Into<String>,
        owned_gift_id: impl Into<String>,
    ) -> Self {
        Self {
            business_connection_id: business_connection_id.into(),
            owned_gift_id: owned_gift_id.into(),
            keep_original_details: None,
            star_count: None,
        }
    }
}

impl AdvancedRequest for AdvancedUpgradeGiftRequest {
    type Response = bool;
    const METHOD: &'static str = "upgradeGift";
}

/// Auto-generated request for `uploadStickerFile`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedUploadStickerFileRequest {
    pub user_id: UserId,
    pub sticker: String,
    pub sticker_format: String,
}

impl AdvancedUploadStickerFileRequest {
    pub fn new(
        user_id: UserId,
        sticker: impl Into<String>,
        sticker_format: impl Into<String>,
    ) -> Self {
        Self {
            user_id,
            sticker: sticker.into(),
            sticker_format: sticker_format.into(),
        }
    }
}

impl AdvancedRequest for AdvancedUploadStickerFileRequest {
    type Response = crate::types::file::File;
    const METHOD: &'static str = "uploadStickerFile";
}

/// Auto-generated request for `verifyChat`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedVerifyChatRequest {
    pub chat_id: ChatId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_description: Option<String>,
}

impl AdvancedVerifyChatRequest {
    pub fn new(chat_id: impl Into<ChatId>) -> Self {
        Self {
            chat_id: chat_id.into(),
            custom_description: None,
        }
    }
}

impl AdvancedRequest for AdvancedVerifyChatRequest {
    type Response = bool;
    const METHOD: &'static str = "verifyChat";
}

/// Auto-generated request for `verifyUser`.
#[derive(Clone, Debug, Serialize)]
pub struct AdvancedVerifyUserRequest {
    pub user_id: UserId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_description: Option<String>,
}

impl AdvancedVerifyUserRequest {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            custom_description: None,
        }
    }
}

impl AdvancedRequest for AdvancedVerifyUserRequest {
    type Response = bool;
    const METHOD: &'static str = "verifyUser";
}
