use serde::de::DeserializeOwned;

use crate::Result;
use crate::types::advanced::*;

#[cfg(feature = "blocking")]
use crate::BlockingClient;
#[cfg(feature = "async")]
use crate::Client;

#[cfg(feature = "async")]
macro_rules! define_async_methods {
    ($(($fn_name:ident, $typed_name:ident, $method:literal, $request_ty:ty)),* $(,)?) => {
        $(
            pub async fn $fn_name<R>(&self, request: &$request_ty) -> Result<R>
            where
                R: DeserializeOwned,
            {
                self.client.call_method($method, request).await
            }

            pub async fn $typed_name(
                &self,
                request: &$request_ty,
            ) -> Result<<$request_ty as AdvancedRequest>::Response> {
                self.call_typed(request).await
            }
        )*
    };
}

#[cfg(feature = "blocking")]
macro_rules! define_blocking_methods {
    ($(($fn_name:ident, $typed_name:ident, $method:literal, $request_ty:ty)),* $(,)?) => {
        $(
            pub fn $fn_name<R>(&self, request: &$request_ty) -> Result<R>
            where
                R: DeserializeOwned,
            {
                self.client.call_method($method, request)
            }

            pub fn $typed_name(
                &self,
                request: &$request_ty,
            ) -> Result<<$request_ty as AdvancedRequest>::Response> {
                self.call_typed(request)
            }
        )*
    };
}

/// Additional Telegram Bot API methods with typed request models.
#[cfg(feature = "async")]
#[derive(Clone)]
pub struct AdvancedService {
    client: Client,
}

#[cfg(feature = "async")]
impl AdvancedService {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Calls advanced methods using request-associated response type.
    pub async fn call_typed<Q>(&self, request: &Q) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        self.client.call_method(Q::METHOD, request).await
    }

    define_async_methods! {
        (add_sticker_to_set, add_sticker_to_set_typed, "addStickerToSet", AdvancedAddStickerToSetRequest),
        (answer_pre_checkout_query, answer_pre_checkout_query_typed, "answerPreCheckoutQuery", AdvancedAnswerPreCheckoutQueryRequest),
        (answer_shipping_query, answer_shipping_query_typed, "answerShippingQuery", AdvancedAnswerShippingQueryRequest),
        (answer_web_app_query, answer_web_app_query_typed, "answerWebAppQuery", AdvancedAnswerWebAppQueryRequest),
        (approve_chat_join_request, approve_chat_join_request_typed, "approveChatJoinRequest", AdvancedApproveChatJoinRequest),
        (approve_suggested_post, approve_suggested_post_typed, "approveSuggestedPost", AdvancedApproveSuggestedPostRequest),
        (close_forum_topic, close_forum_topic_typed, "closeForumTopic", AdvancedCloseForumTopicRequest),
        (close_general_forum_topic, close_general_forum_topic_typed, "closeGeneralForumTopic", AdvancedCloseGeneralForumTopicRequest),
        (convert_gift_to_stars, convert_gift_to_stars_typed, "convertGiftToStars", AdvancedConvertGiftToStarsRequest),
        (create_chat_subscription_invite_link, create_chat_subscription_invite_link_typed, "createChatSubscriptionInviteLink", AdvancedCreateChatSubscriptionInviteLinkRequest),
        (create_forum_topic, create_forum_topic_typed, "createForumTopic", AdvancedCreateForumTopicRequest),
        (create_invoice_link, create_invoice_link_typed, "createInvoiceLink", AdvancedCreateInvoiceLinkRequest),
        (create_new_sticker_set, create_new_sticker_set_typed, "createNewStickerSet", AdvancedCreateNewStickerSetRequest),
        (decline_chat_join_request, decline_chat_join_request_typed, "declineChatJoinRequest", AdvancedDeclineChatJoinRequest),
        (decline_suggested_post, decline_suggested_post_typed, "declineSuggestedPost", AdvancedDeclineSuggestedPostRequest),
        (delete_business_messages, delete_business_messages_typed, "deleteBusinessMessages", AdvancedDeleteBusinessMessagesRequest),
        (delete_forum_topic, delete_forum_topic_typed, "deleteForumTopic", AdvancedDeleteForumTopicRequest),
        (delete_sticker_from_set, delete_sticker_from_set_typed, "deleteStickerFromSet", AdvancedDeleteStickerFromSetRequest),
        (delete_sticker_set, delete_sticker_set_typed, "deleteStickerSet", AdvancedDeleteStickerSetRequest),
        (delete_story, delete_story_typed, "deleteStory", AdvancedDeleteStoryRequest),
        (edit_chat_subscription_invite_link, edit_chat_subscription_invite_link_typed, "editChatSubscriptionInviteLink", AdvancedEditChatSubscriptionInviteLinkRequest),
        (edit_forum_topic, edit_forum_topic_typed, "editForumTopic", AdvancedEditForumTopicRequest),
        (edit_general_forum_topic, edit_general_forum_topic_typed, "editGeneralForumTopic", AdvancedEditGeneralForumTopicRequest),
        (edit_message_checklist, edit_message_checklist_typed, "editMessageChecklist", AdvancedEditMessageChecklistRequest),
        (edit_message_media, edit_message_media_typed, "editMessageMedia", AdvancedEditMessageMediaRequest),
        (edit_story, edit_story_typed, "editStory", AdvancedEditStoryRequest),
        (edit_user_star_subscription, edit_user_star_subscription_typed, "editUserStarSubscription", AdvancedEditUserStarSubscriptionRequest),
        (forward_messages, forward_messages_typed, "forwardMessages", AdvancedForwardMessagesRequest),
        (get_available_gifts, get_available_gifts_typed, "getAvailableGifts", AdvancedGetAvailableGiftsRequest),
        (get_business_account_gifts, get_business_account_gifts_typed, "getBusinessAccountGifts", AdvancedGetBusinessAccountGiftsRequest),
        (get_business_account_star_balance, get_business_account_star_balance_typed, "getBusinessAccountStarBalance", AdvancedGetBusinessAccountStarBalanceRequest),
        (get_business_connection, get_business_connection_typed, "getBusinessConnection", AdvancedGetBusinessConnectionRequest),
        (get_chat_gifts, get_chat_gifts_typed, "getChatGifts", AdvancedGetChatGiftsRequest),
        (get_chat_menu_button, get_chat_menu_button_typed, "getChatMenuButton", AdvancedGetChatMenuButtonRequest),
        (get_custom_emoji_stickers, get_custom_emoji_stickers_typed, "getCustomEmojiStickers", AdvancedGetCustomEmojiStickersRequest),
        (get_forum_topic_icon_stickers, get_forum_topic_icon_stickers_typed, "getForumTopicIconStickers", AdvancedGetForumTopicIconStickersRequest),
        (get_game_high_scores, get_game_high_scores_typed, "getGameHighScores", AdvancedGetGameHighScoresRequest),
        (get_my_default_administrator_rights, get_my_default_administrator_rights_typed, "getMyDefaultAdministratorRights", AdvancedGetMyDefaultAdministratorRightsRequest),
        (get_my_star_balance, get_my_star_balance_typed, "getMyStarBalance", AdvancedGetMyStarBalanceRequest),
        (get_star_transactions, get_star_transactions_typed, "getStarTransactions", AdvancedGetStarTransactionsRequest),
        (get_sticker_set, get_sticker_set_typed, "getStickerSet", AdvancedGetStickerSetRequest),
        (get_user_chat_boosts, get_user_chat_boosts_typed, "getUserChatBoosts", AdvancedGetUserChatBoostsRequest),
        (get_user_gifts, get_user_gifts_typed, "getUserGifts", AdvancedGetUserGiftsRequest),
        (get_user_profile_audios, get_user_profile_audios_typed, "getUserProfileAudios", AdvancedGetUserProfileAudiosRequest),
        (gift_premium_subscription, gift_premium_subscription_typed, "giftPremiumSubscription", AdvancedGiftPremiumSubscriptionRequest),
        (hide_general_forum_topic, hide_general_forum_topic_typed, "hideGeneralForumTopic", AdvancedHideGeneralForumTopicRequest),
        (post_story, post_story_typed, "postStory", AdvancedPostStoryRequest),
        (read_business_message, read_business_message_typed, "readBusinessMessage", AdvancedReadBusinessMessageRequest),
        (refund_star_payment, refund_star_payment_typed, "refundStarPayment", AdvancedRefundStarPaymentRequest),
        (remove_business_account_profile_photo, remove_business_account_profile_photo_typed, "removeBusinessAccountProfilePhoto", AdvancedRemoveBusinessAccountProfilePhotoRequest),
        (remove_chat_verification, remove_chat_verification_typed, "removeChatVerification", AdvancedRemoveChatVerificationRequest),
        (remove_my_profile_photo, remove_my_profile_photo_typed, "removeMyProfilePhoto", AdvancedRemoveMyProfilePhotoRequest),
        (remove_user_verification, remove_user_verification_typed, "removeUserVerification", AdvancedRemoveUserVerificationRequest),
        (reopen_forum_topic, reopen_forum_topic_typed, "reopenForumTopic", AdvancedReopenForumTopicRequest),
        (reopen_general_forum_topic, reopen_general_forum_topic_typed, "reopenGeneralForumTopic", AdvancedReopenGeneralForumTopicRequest),
        (replace_sticker_in_set, replace_sticker_in_set_typed, "replaceStickerInSet", AdvancedReplaceStickerInSetRequest),
        (repost_story, repost_story_typed, "repostStory", AdvancedRepostStoryRequest),
        (save_prepared_inline_message, save_prepared_inline_message_typed, "savePreparedInlineMessage", AdvancedSavePreparedInlineMessageRequest),
        (send_checklist, send_checklist_typed, "sendChecklist", AdvancedSendChecklistRequest),
        (send_game, send_game_typed, "sendGame", AdvancedSendGameRequest),
        (send_gift, send_gift_typed, "sendGift", AdvancedSendGiftRequest),
        (send_invoice, send_invoice_typed, "sendInvoice", AdvancedSendInvoiceRequest),
        (send_message_draft, send_message_draft_typed, "sendMessageDraft", AdvancedSendMessageDraftRequest),
        (send_paid_media, send_paid_media_typed, "sendPaidMedia", AdvancedSendPaidMediaRequest),
        (send_sticker, send_sticker_typed, "sendSticker", AdvancedSendStickerRequest),
        (set_business_account_bio, set_business_account_bio_typed, "setBusinessAccountBio", AdvancedSetBusinessAccountBioRequest),
        (set_business_account_gift_settings, set_business_account_gift_settings_typed, "setBusinessAccountGiftSettings", AdvancedSetBusinessAccountGiftSettingsRequest),
        (set_business_account_name, set_business_account_name_typed, "setBusinessAccountName", AdvancedSetBusinessAccountNameRequest),
        (set_business_account_profile_photo, set_business_account_profile_photo_typed, "setBusinessAccountProfilePhoto", AdvancedSetBusinessAccountProfilePhotoRequest),
        (set_business_account_username, set_business_account_username_typed, "setBusinessAccountUsername", AdvancedSetBusinessAccountUsernameRequest),
        (set_chat_menu_button, set_chat_menu_button_typed, "setChatMenuButton", AdvancedSetChatMenuButtonRequest),
        (set_chat_photo, set_chat_photo_typed, "setChatPhoto", AdvancedSetChatPhotoRequest),
        (set_custom_emoji_sticker_set_thumbnail, set_custom_emoji_sticker_set_thumbnail_typed, "setCustomEmojiStickerSetThumbnail", AdvancedSetCustomEmojiStickerSetThumbnailRequest),
        (set_game_score, set_game_score_typed, "setGameScore", AdvancedSetGameScoreRequest),
        (set_message_reaction, set_message_reaction_typed, "setMessageReaction", AdvancedSetMessageReactionRequest),
        (set_my_default_administrator_rights, set_my_default_administrator_rights_typed, "setMyDefaultAdministratorRights", AdvancedSetMyDefaultAdministratorRightsRequest),
        (set_my_profile_photo, set_my_profile_photo_typed, "setMyProfilePhoto", AdvancedSetMyProfilePhotoRequest),
        (set_passport_data_errors, set_passport_data_errors_typed, "setPassportDataErrors", AdvancedSetPassportDataErrorsRequest),
        (set_sticker_emoji_list, set_sticker_emoji_list_typed, "setStickerEmojiList", AdvancedSetStickerEmojiListRequest),
        (set_sticker_keywords, set_sticker_keywords_typed, "setStickerKeywords", AdvancedSetStickerKeywordsRequest),
        (set_sticker_mask_position, set_sticker_mask_position_typed, "setStickerMaskPosition", AdvancedSetStickerMaskPositionRequest),
        (set_sticker_position_in_set, set_sticker_position_in_set_typed, "setStickerPositionInSet", AdvancedSetStickerPositionInSetRequest),
        (set_sticker_set_thumbnail, set_sticker_set_thumbnail_typed, "setStickerSetThumbnail", AdvancedSetStickerSetThumbnailRequest),
        (set_sticker_set_title, set_sticker_set_title_typed, "setStickerSetTitle", AdvancedSetStickerSetTitleRequest),
        (set_user_emoji_status, set_user_emoji_status_typed, "setUserEmojiStatus", AdvancedSetUserEmojiStatusRequest),
        (transfer_business_account_stars, transfer_business_account_stars_typed, "transferBusinessAccountStars", AdvancedTransferBusinessAccountStarsRequest),
        (transfer_gift, transfer_gift_typed, "transferGift", AdvancedTransferGiftRequest),
        (unhide_general_forum_topic, unhide_general_forum_topic_typed, "unhideGeneralForumTopic", AdvancedUnhideGeneralForumTopicRequest),
        (unpin_all_forum_topic_messages, unpin_all_forum_topic_messages_typed, "unpinAllForumTopicMessages", AdvancedUnpinAllForumTopicMessagesRequest),
        (unpin_all_general_forum_topic_messages, unpin_all_general_forum_topic_messages_typed, "unpinAllGeneralForumTopicMessages", AdvancedUnpinAllGeneralForumTopicMessagesRequest),
        (upgrade_gift, upgrade_gift_typed, "upgradeGift", AdvancedUpgradeGiftRequest),
        (upload_sticker_file, upload_sticker_file_typed, "uploadStickerFile", AdvancedUploadStickerFileRequest),
        (verify_chat, verify_chat_typed, "verifyChat", AdvancedVerifyChatRequest),
        (verify_user, verify_user_typed, "verifyUser", AdvancedVerifyUserRequest),
    }
}

/// Blocking additional Telegram Bot API methods with typed request models.
#[cfg(feature = "blocking")]
#[derive(Clone)]
pub struct BlockingAdvancedService {
    client: BlockingClient,
}

#[cfg(feature = "blocking")]
impl BlockingAdvancedService {
    pub(crate) fn new(client: BlockingClient) -> Self {
        Self { client }
    }

    /// Calls advanced methods using request-associated response type.
    pub fn call_typed<Q>(&self, request: &Q) -> Result<Q::Response>
    where
        Q: AdvancedRequest,
    {
        self.client.call_method(Q::METHOD, request)
    }

    define_blocking_methods! {
        (add_sticker_to_set, add_sticker_to_set_typed, "addStickerToSet", AdvancedAddStickerToSetRequest),
        (answer_pre_checkout_query, answer_pre_checkout_query_typed, "answerPreCheckoutQuery", AdvancedAnswerPreCheckoutQueryRequest),
        (answer_shipping_query, answer_shipping_query_typed, "answerShippingQuery", AdvancedAnswerShippingQueryRequest),
        (answer_web_app_query, answer_web_app_query_typed, "answerWebAppQuery", AdvancedAnswerWebAppQueryRequest),
        (approve_chat_join_request, approve_chat_join_request_typed, "approveChatJoinRequest", AdvancedApproveChatJoinRequest),
        (approve_suggested_post, approve_suggested_post_typed, "approveSuggestedPost", AdvancedApproveSuggestedPostRequest),
        (close_forum_topic, close_forum_topic_typed, "closeForumTopic", AdvancedCloseForumTopicRequest),
        (close_general_forum_topic, close_general_forum_topic_typed, "closeGeneralForumTopic", AdvancedCloseGeneralForumTopicRequest),
        (convert_gift_to_stars, convert_gift_to_stars_typed, "convertGiftToStars", AdvancedConvertGiftToStarsRequest),
        (create_chat_subscription_invite_link, create_chat_subscription_invite_link_typed, "createChatSubscriptionInviteLink", AdvancedCreateChatSubscriptionInviteLinkRequest),
        (create_forum_topic, create_forum_topic_typed, "createForumTopic", AdvancedCreateForumTopicRequest),
        (create_invoice_link, create_invoice_link_typed, "createInvoiceLink", AdvancedCreateInvoiceLinkRequest),
        (create_new_sticker_set, create_new_sticker_set_typed, "createNewStickerSet", AdvancedCreateNewStickerSetRequest),
        (decline_chat_join_request, decline_chat_join_request_typed, "declineChatJoinRequest", AdvancedDeclineChatJoinRequest),
        (decline_suggested_post, decline_suggested_post_typed, "declineSuggestedPost", AdvancedDeclineSuggestedPostRequest),
        (delete_business_messages, delete_business_messages_typed, "deleteBusinessMessages", AdvancedDeleteBusinessMessagesRequest),
        (delete_forum_topic, delete_forum_topic_typed, "deleteForumTopic", AdvancedDeleteForumTopicRequest),
        (delete_sticker_from_set, delete_sticker_from_set_typed, "deleteStickerFromSet", AdvancedDeleteStickerFromSetRequest),
        (delete_sticker_set, delete_sticker_set_typed, "deleteStickerSet", AdvancedDeleteStickerSetRequest),
        (delete_story, delete_story_typed, "deleteStory", AdvancedDeleteStoryRequest),
        (edit_chat_subscription_invite_link, edit_chat_subscription_invite_link_typed, "editChatSubscriptionInviteLink", AdvancedEditChatSubscriptionInviteLinkRequest),
        (edit_forum_topic, edit_forum_topic_typed, "editForumTopic", AdvancedEditForumTopicRequest),
        (edit_general_forum_topic, edit_general_forum_topic_typed, "editGeneralForumTopic", AdvancedEditGeneralForumTopicRequest),
        (edit_message_checklist, edit_message_checklist_typed, "editMessageChecklist", AdvancedEditMessageChecklistRequest),
        (edit_message_media, edit_message_media_typed, "editMessageMedia", AdvancedEditMessageMediaRequest),
        (edit_story, edit_story_typed, "editStory", AdvancedEditStoryRequest),
        (edit_user_star_subscription, edit_user_star_subscription_typed, "editUserStarSubscription", AdvancedEditUserStarSubscriptionRequest),
        (forward_messages, forward_messages_typed, "forwardMessages", AdvancedForwardMessagesRequest),
        (get_available_gifts, get_available_gifts_typed, "getAvailableGifts", AdvancedGetAvailableGiftsRequest),
        (get_business_account_gifts, get_business_account_gifts_typed, "getBusinessAccountGifts", AdvancedGetBusinessAccountGiftsRequest),
        (get_business_account_star_balance, get_business_account_star_balance_typed, "getBusinessAccountStarBalance", AdvancedGetBusinessAccountStarBalanceRequest),
        (get_business_connection, get_business_connection_typed, "getBusinessConnection", AdvancedGetBusinessConnectionRequest),
        (get_chat_gifts, get_chat_gifts_typed, "getChatGifts", AdvancedGetChatGiftsRequest),
        (get_chat_menu_button, get_chat_menu_button_typed, "getChatMenuButton", AdvancedGetChatMenuButtonRequest),
        (get_custom_emoji_stickers, get_custom_emoji_stickers_typed, "getCustomEmojiStickers", AdvancedGetCustomEmojiStickersRequest),
        (get_forum_topic_icon_stickers, get_forum_topic_icon_stickers_typed, "getForumTopicIconStickers", AdvancedGetForumTopicIconStickersRequest),
        (get_game_high_scores, get_game_high_scores_typed, "getGameHighScores", AdvancedGetGameHighScoresRequest),
        (get_my_default_administrator_rights, get_my_default_administrator_rights_typed, "getMyDefaultAdministratorRights", AdvancedGetMyDefaultAdministratorRightsRequest),
        (get_my_star_balance, get_my_star_balance_typed, "getMyStarBalance", AdvancedGetMyStarBalanceRequest),
        (get_star_transactions, get_star_transactions_typed, "getStarTransactions", AdvancedGetStarTransactionsRequest),
        (get_sticker_set, get_sticker_set_typed, "getStickerSet", AdvancedGetStickerSetRequest),
        (get_user_chat_boosts, get_user_chat_boosts_typed, "getUserChatBoosts", AdvancedGetUserChatBoostsRequest),
        (get_user_gifts, get_user_gifts_typed, "getUserGifts", AdvancedGetUserGiftsRequest),
        (get_user_profile_audios, get_user_profile_audios_typed, "getUserProfileAudios", AdvancedGetUserProfileAudiosRequest),
        (gift_premium_subscription, gift_premium_subscription_typed, "giftPremiumSubscription", AdvancedGiftPremiumSubscriptionRequest),
        (hide_general_forum_topic, hide_general_forum_topic_typed, "hideGeneralForumTopic", AdvancedHideGeneralForumTopicRequest),
        (post_story, post_story_typed, "postStory", AdvancedPostStoryRequest),
        (read_business_message, read_business_message_typed, "readBusinessMessage", AdvancedReadBusinessMessageRequest),
        (refund_star_payment, refund_star_payment_typed, "refundStarPayment", AdvancedRefundStarPaymentRequest),
        (remove_business_account_profile_photo, remove_business_account_profile_photo_typed, "removeBusinessAccountProfilePhoto", AdvancedRemoveBusinessAccountProfilePhotoRequest),
        (remove_chat_verification, remove_chat_verification_typed, "removeChatVerification", AdvancedRemoveChatVerificationRequest),
        (remove_my_profile_photo, remove_my_profile_photo_typed, "removeMyProfilePhoto", AdvancedRemoveMyProfilePhotoRequest),
        (remove_user_verification, remove_user_verification_typed, "removeUserVerification", AdvancedRemoveUserVerificationRequest),
        (reopen_forum_topic, reopen_forum_topic_typed, "reopenForumTopic", AdvancedReopenForumTopicRequest),
        (reopen_general_forum_topic, reopen_general_forum_topic_typed, "reopenGeneralForumTopic", AdvancedReopenGeneralForumTopicRequest),
        (replace_sticker_in_set, replace_sticker_in_set_typed, "replaceStickerInSet", AdvancedReplaceStickerInSetRequest),
        (repost_story, repost_story_typed, "repostStory", AdvancedRepostStoryRequest),
        (save_prepared_inline_message, save_prepared_inline_message_typed, "savePreparedInlineMessage", AdvancedSavePreparedInlineMessageRequest),
        (send_checklist, send_checklist_typed, "sendChecklist", AdvancedSendChecklistRequest),
        (send_game, send_game_typed, "sendGame", AdvancedSendGameRequest),
        (send_gift, send_gift_typed, "sendGift", AdvancedSendGiftRequest),
        (send_invoice, send_invoice_typed, "sendInvoice", AdvancedSendInvoiceRequest),
        (send_message_draft, send_message_draft_typed, "sendMessageDraft", AdvancedSendMessageDraftRequest),
        (send_paid_media, send_paid_media_typed, "sendPaidMedia", AdvancedSendPaidMediaRequest),
        (send_sticker, send_sticker_typed, "sendSticker", AdvancedSendStickerRequest),
        (set_business_account_bio, set_business_account_bio_typed, "setBusinessAccountBio", AdvancedSetBusinessAccountBioRequest),
        (set_business_account_gift_settings, set_business_account_gift_settings_typed, "setBusinessAccountGiftSettings", AdvancedSetBusinessAccountGiftSettingsRequest),
        (set_business_account_name, set_business_account_name_typed, "setBusinessAccountName", AdvancedSetBusinessAccountNameRequest),
        (set_business_account_profile_photo, set_business_account_profile_photo_typed, "setBusinessAccountProfilePhoto", AdvancedSetBusinessAccountProfilePhotoRequest),
        (set_business_account_username, set_business_account_username_typed, "setBusinessAccountUsername", AdvancedSetBusinessAccountUsernameRequest),
        (set_chat_menu_button, set_chat_menu_button_typed, "setChatMenuButton", AdvancedSetChatMenuButtonRequest),
        (set_chat_photo, set_chat_photo_typed, "setChatPhoto", AdvancedSetChatPhotoRequest),
        (set_custom_emoji_sticker_set_thumbnail, set_custom_emoji_sticker_set_thumbnail_typed, "setCustomEmojiStickerSetThumbnail", AdvancedSetCustomEmojiStickerSetThumbnailRequest),
        (set_game_score, set_game_score_typed, "setGameScore", AdvancedSetGameScoreRequest),
        (set_message_reaction, set_message_reaction_typed, "setMessageReaction", AdvancedSetMessageReactionRequest),
        (set_my_default_administrator_rights, set_my_default_administrator_rights_typed, "setMyDefaultAdministratorRights", AdvancedSetMyDefaultAdministratorRightsRequest),
        (set_my_profile_photo, set_my_profile_photo_typed, "setMyProfilePhoto", AdvancedSetMyProfilePhotoRequest),
        (set_passport_data_errors, set_passport_data_errors_typed, "setPassportDataErrors", AdvancedSetPassportDataErrorsRequest),
        (set_sticker_emoji_list, set_sticker_emoji_list_typed, "setStickerEmojiList", AdvancedSetStickerEmojiListRequest),
        (set_sticker_keywords, set_sticker_keywords_typed, "setStickerKeywords", AdvancedSetStickerKeywordsRequest),
        (set_sticker_mask_position, set_sticker_mask_position_typed, "setStickerMaskPosition", AdvancedSetStickerMaskPositionRequest),
        (set_sticker_position_in_set, set_sticker_position_in_set_typed, "setStickerPositionInSet", AdvancedSetStickerPositionInSetRequest),
        (set_sticker_set_thumbnail, set_sticker_set_thumbnail_typed, "setStickerSetThumbnail", AdvancedSetStickerSetThumbnailRequest),
        (set_sticker_set_title, set_sticker_set_title_typed, "setStickerSetTitle", AdvancedSetStickerSetTitleRequest),
        (set_user_emoji_status, set_user_emoji_status_typed, "setUserEmojiStatus", AdvancedSetUserEmojiStatusRequest),
        (transfer_business_account_stars, transfer_business_account_stars_typed, "transferBusinessAccountStars", AdvancedTransferBusinessAccountStarsRequest),
        (transfer_gift, transfer_gift_typed, "transferGift", AdvancedTransferGiftRequest),
        (unhide_general_forum_topic, unhide_general_forum_topic_typed, "unhideGeneralForumTopic", AdvancedUnhideGeneralForumTopicRequest),
        (unpin_all_forum_topic_messages, unpin_all_forum_topic_messages_typed, "unpinAllForumTopicMessages", AdvancedUnpinAllForumTopicMessagesRequest),
        (unpin_all_general_forum_topic_messages, unpin_all_general_forum_topic_messages_typed, "unpinAllGeneralForumTopicMessages", AdvancedUnpinAllGeneralForumTopicMessagesRequest),
        (upgrade_gift, upgrade_gift_typed, "upgradeGift", AdvancedUpgradeGiftRequest),
        (upload_sticker_file, upload_sticker_file_typed, "uploadStickerFile", AdvancedUploadStickerFileRequest),
        (verify_chat, verify_chat_typed, "verifyChat", AdvancedVerifyChatRequest),
        (verify_user, verify_user_typed, "verifyUser", AdvancedVerifyUserRequest),
    }
}
