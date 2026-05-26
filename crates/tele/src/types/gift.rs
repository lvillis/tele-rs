use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::types::bot::User;
use crate::types::message::Chat;
use crate::types::message::{MessageEntity, PaidMedia};
use crate::types::sticker::Sticker;
use crate::types::tagged::{strip_type, tagged_kind};

/// Origin of a unique gift service message.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum UniqueGiftOrigin {
    Upgrade,
    Transfer,
    Resale,
    GiftedUpgrade,
    Offer,
    Unknown(String),
}

impl UniqueGiftOrigin {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Upgrade => "upgrade",
            Self::Transfer => "transfer",
            Self::Resale => "resale",
            Self::GiftedUpgrade => "gifted_upgrade",
            Self::Offer => "offer",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for UniqueGiftOrigin {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "upgrade" => Self::Upgrade,
            "transfer" => Self::Transfer,
            "resale" => Self::Resale,
            "gifted_upgrade" => Self::GiftedUpgrade,
            "offer" => Self::Offer,
            _ => Self::Unknown(value),
        })
    }
}

/// Telegram gift background.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct GiftBackground {
    pub center_color: u32,
    pub edge_color: u32,
    pub text_color: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram gift that can be sent by the bot.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Gift {
    pub id: String,
    pub sticker: Sticker,
    pub star_count: i64,
    #[serde(default)]
    pub upgrade_star_count: Option<i64>,
    #[serde(default)]
    pub is_premium: bool,
    #[serde(default)]
    pub has_colors: bool,
    #[serde(default)]
    pub total_count: Option<i64>,
    #[serde(default)]
    pub remaining_count: Option<i64>,
    #[serde(default)]
    pub personal_total_count: Option<i64>,
    #[serde(default)]
    pub personal_remaining_count: Option<i64>,
    #[serde(default)]
    pub background: Option<GiftBackground>,
    #[serde(default)]
    pub unique_gift_variant_count: Option<i64>,
    #[serde(default)]
    pub publisher_chat: Option<Chat>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram list of gifts.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Gifts {
    pub gifts: Vec<Gift>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Model of a unique gift.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UniqueGiftModel {
    pub name: String,
    pub sticker: Sticker,
    pub rarity_per_mille: u32,
    #[serde(default)]
    pub rarity: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Symbol of a unique gift.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UniqueGiftSymbol {
    pub name: String,
    pub sticker: Sticker,
    pub rarity_per_mille: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// RGB color palette of a unique gift backdrop.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UniqueGiftBackdropColors {
    pub center_color: u32,
    pub edge_color: u32,
    pub symbol_color: u32,
    pub text_color: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Backdrop of a unique gift.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UniqueGiftBackdrop {
    pub name: String,
    pub colors: UniqueGiftBackdropColors,
    pub rarity_per_mille: u32,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Chat appearance colors unlocked by a unique gift.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UniqueGiftColors {
    pub model_custom_emoji_id: String,
    pub symbol_custom_emoji_id: String,
    pub light_theme_main_color: u32,
    pub light_theme_other_colors: Vec<u32>,
    pub dark_theme_main_color: u32,
    pub dark_theme_other_colors: Vec<u32>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram unique gift.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UniqueGift {
    pub gift_id: String,
    pub base_name: String,
    pub name: String,
    pub number: i64,
    pub model: UniqueGiftModel,
    pub symbol: UniqueGiftSymbol,
    pub backdrop: UniqueGiftBackdrop,
    #[serde(default)]
    pub is_premium: bool,
    #[serde(default)]
    pub is_burned: bool,
    #[serde(default)]
    pub is_from_blockchain: bool,
    #[serde(default)]
    pub colors: Option<UniqueGiftColors>,
    #[serde(default)]
    pub publisher_chat: Option<Chat>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Service message payload for a regular gift.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct GiftInfo {
    pub gift: Gift,
    #[serde(default)]
    pub owned_gift_id: Option<String>,
    #[serde(default)]
    pub convert_star_count: Option<i64>,
    #[serde(default)]
    pub prepaid_upgrade_star_count: Option<i64>,
    #[serde(default)]
    pub is_upgrade_separate: bool,
    #[serde(default)]
    pub can_be_upgraded: bool,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub entities: Option<Vec<MessageEntity>>,
    #[serde(default)]
    pub is_private: bool,
    #[serde(default)]
    pub unique_gift_number: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Service message payload for a unique gift.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct UniqueGiftInfo {
    pub gift: UniqueGift,
    pub origin: UniqueGiftOrigin,
    #[serde(default)]
    pub last_resale_currency: Option<String>,
    #[serde(default)]
    pub last_resale_amount: Option<i64>,
    #[serde(default)]
    pub owned_gift_id: Option<String>,
    #[serde(default)]
    pub transfer_star_count: Option<i64>,
    #[serde(default)]
    pub next_transfer_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Regular gift owned by a user or a chat.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct OwnedGiftRegular {
    pub gift: Gift,
    #[serde(default)]
    pub owned_gift_id: Option<String>,
    #[serde(default)]
    pub sender_user: Option<User>,
    pub send_date: i64,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub entities: Option<Vec<MessageEntity>>,
    #[serde(default)]
    pub is_private: bool,
    #[serde(default)]
    pub is_saved: bool,
    #[serde(default)]
    pub can_be_upgraded: bool,
    #[serde(default)]
    pub was_refunded: bool,
    #[serde(default)]
    pub convert_star_count: Option<i64>,
    #[serde(default)]
    pub prepaid_upgrade_star_count: Option<i64>,
    #[serde(default)]
    pub is_upgrade_separate: bool,
    #[serde(default)]
    pub unique_gift_number: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Unique gift owned by a user or a chat.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct OwnedGiftUnique {
    pub gift: UniqueGift,
    #[serde(default)]
    pub owned_gift_id: Option<String>,
    #[serde(default)]
    pub sender_user: Option<User>,
    pub send_date: i64,
    #[serde(default)]
    pub is_saved: bool,
    #[serde(default)]
    pub can_be_transferred: bool,
    #[serde(default)]
    pub transfer_star_count: Option<i64>,
    #[serde(default)]
    pub next_transfer_date: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Gift owned by a user or a chat.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum OwnedGift {
    Regular(Box<OwnedGiftRegular>),
    Unique(Box<OwnedGiftUnique>),
    Unknown(Value),
}

impl OwnedGift {
    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Regular(_) => Some("regular"),
            Self::Unique(_) => Some("unique"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn is_regular(&self) -> bool {
        matches!(self, Self::Regular(_))
    }

    pub fn is_unique(&self) -> bool {
        matches!(self, Self::Unique(_))
    }

    pub fn as_regular(&self) -> Option<&OwnedGiftRegular> {
        match self {
            Self::Regular(value) => Some(value),
            Self::Unique(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_regular(self) -> Option<OwnedGiftRegular> {
        match self {
            Self::Regular(value) => Some(*value),
            Self::Unique(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_unique(&self) -> Option<&OwnedGiftUnique> {
        match self {
            Self::Unique(value) => Some(value),
            Self::Regular(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_unique(self) -> Option<OwnedGiftUnique> {
        match self {
            Self::Unique(value) => Some(*value),
            Self::Regular(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_unknown_value(&self) -> Option<&Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Regular(_) | Self::Unique(_) => None,
        }
    }

    pub fn into_unknown_value(self) -> Option<Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Regular(_) | Self::Unique(_) => None,
        }
    }
}

impl<'de> Deserialize<'de> for OwnedGift {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match tagged_kind(&value) {
            Some("regular") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::Regular)
                .map_err(serde::de::Error::custom),
            Some("unique") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::Unique)
                .map_err(serde::de::Error::custom),
            Some(_) | None => Ok(Self::Unknown(value)),
        }
    }
}

/// Telegram list of gifts owned by a user or a chat.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct OwnedGifts {
    pub total_count: u64,
    pub gifts: Vec<OwnedGift>,
    #[serde(default)]
    pub next_offset: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Type of transaction with a user.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum TransactionKind {
    InvoicePayment,
    PaidMediaPayment,
    GiftPurchase,
    PremiumPurchase,
    BusinessAccountTransfer,
    Unknown(String),
}

impl TransactionKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::InvoicePayment => "invoice_payment",
            Self::PaidMediaPayment => "paid_media_payment",
            Self::GiftPurchase => "gift_purchase",
            Self::PremiumPurchase => "premium_purchase",
            Self::BusinessAccountTransfer => "business_account_transfer",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

impl<'de> Deserialize<'de> for TransactionKind {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "invoice_payment" => Self::InvoicePayment,
            "paid_media_payment" => Self::PaidMediaPayment,
            "gift_purchase" => Self::GiftPurchase,
            "premium_purchase" => Self::PremiumPurchase,
            "business_account_transfer" => Self::BusinessAccountTransfer,
            _ => Self::Unknown(value),
        })
    }
}

/// State of a revenue withdrawal operation.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum RevenueWithdrawalState {
    Pending(RevenueWithdrawalStatePending),
    Succeeded(Box<RevenueWithdrawalStateSucceeded>),
    Failed(RevenueWithdrawalStateFailed),
    Unknown(Value),
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct RevenueWithdrawalStatePending {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct RevenueWithdrawalStateSucceeded {
    pub date: i64,
    pub url: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct RevenueWithdrawalStateFailed {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl RevenueWithdrawalState {
    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Pending(_) => Some("pending"),
            Self::Succeeded(_) => Some("succeeded"),
            Self::Failed(_) => Some("failed"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn as_pending(&self) -> Option<&RevenueWithdrawalStatePending> {
        match self {
            Self::Pending(value) => Some(value),
            Self::Succeeded(_) | Self::Failed(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_pending(self) -> Option<RevenueWithdrawalStatePending> {
        match self {
            Self::Pending(value) => Some(value),
            Self::Succeeded(_) | Self::Failed(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_succeeded(&self) -> Option<&RevenueWithdrawalStateSucceeded> {
        match self {
            Self::Succeeded(value) => Some(value),
            Self::Pending(_) | Self::Failed(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_succeeded(self) -> Option<RevenueWithdrawalStateSucceeded> {
        match self {
            Self::Succeeded(value) => Some(*value),
            Self::Pending(_) | Self::Failed(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_failed(&self) -> Option<&RevenueWithdrawalStateFailed> {
        match self {
            Self::Failed(value) => Some(value),
            Self::Pending(_) | Self::Succeeded(_) | Self::Unknown(_) => None,
        }
    }

    pub fn into_failed(self) -> Option<RevenueWithdrawalStateFailed> {
        match self {
            Self::Failed(value) => Some(value),
            Self::Pending(_) | Self::Succeeded(_) | Self::Unknown(_) => None,
        }
    }

    pub fn as_unknown_value(&self) -> Option<&Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Pending(_) | Self::Succeeded(_) | Self::Failed(_) => None,
        }
    }

    pub fn into_unknown_value(self) -> Option<Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::Pending(_) | Self::Succeeded(_) | Self::Failed(_) => None,
        }
    }
}

impl<'de> Deserialize<'de> for RevenueWithdrawalState {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match tagged_kind(&value) {
            Some("pending") => serde_json::from_value(strip_type(value))
                .map(Self::Pending)
                .map_err(serde::de::Error::custom),
            Some("succeeded") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::Succeeded)
                .map_err(serde::de::Error::custom),
            Some("failed") => serde_json::from_value(strip_type(value))
                .map(Self::Failed)
                .map_err(serde::de::Error::custom),
            Some(_) | None => Ok(Self::Unknown(value)),
        }
    }
}

/// Affiliate commission details for a Star transaction.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct AffiliateInfo {
    #[serde(default)]
    pub affiliate_user: Option<User>,
    #[serde(default)]
    pub affiliate_chat: Option<Chat>,
    pub commission_per_mille: i64,
    pub amount: i64,
    #[serde(default)]
    pub nanostar_amount: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Star transaction partner represented by a user.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct TransactionPartnerUser {
    pub transaction_type: TransactionKind,
    pub user: User,
    #[serde(default)]
    pub affiliate: Option<AffiliateInfo>,
    #[serde(default)]
    pub invoice_payload: Option<String>,
    #[serde(default)]
    pub subscription_period: Option<i64>,
    #[serde(default)]
    pub paid_media: Option<Vec<PaidMedia>>,
    #[serde(default)]
    pub paid_media_payload: Option<String>,
    #[serde(default)]
    pub gift: Option<Box<Gift>>,
    #[serde(default)]
    pub premium_subscription_duration: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Star transaction partner represented by a chat.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct TransactionPartnerChat {
    pub chat: Chat,
    #[serde(default)]
    pub gift: Option<Box<Gift>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Star transaction partner represented by an affiliate program.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct TransactionPartnerAffiliateProgram {
    #[serde(default)]
    pub sponsor_user: Option<User>,
    pub commission_per_mille: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Star transaction partner represented by Fragment.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct TransactionPartnerFragment {
    #[serde(default)]
    pub withdrawal_state: Option<RevenueWithdrawalState>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Star transaction partner represented by Telegram Ads.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct TransactionPartnerTelegramAds {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Star transaction partner represented by paid Bot API broadcasting.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct TransactionPartnerTelegramApi {
    pub request_count: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Star transaction partner represented by an unknown source or recipient.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct TransactionPartnerOther {
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Source or recipient of a Star transaction.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum TransactionPartner {
    User(Box<TransactionPartnerUser>),
    Chat(Box<TransactionPartnerChat>),
    AffiliateProgram(Box<TransactionPartnerAffiliateProgram>),
    Fragment(Box<TransactionPartnerFragment>),
    TelegramAds(TransactionPartnerTelegramAds),
    TelegramApi(Box<TransactionPartnerTelegramApi>),
    Other(TransactionPartnerOther),
    Unknown(Value),
}

impl TransactionPartner {
    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::User(_) => Some("user"),
            Self::Chat(_) => Some("chat"),
            Self::AffiliateProgram(_) => Some("affiliate_program"),
            Self::Fragment(_) => Some("fragment"),
            Self::TelegramAds(_) => Some("telegram_ads"),
            Self::TelegramApi(_) => Some("telegram_api"),
            Self::Other(_) => Some("other"),
            Self::Unknown(value) => tagged_kind(value),
        }
    }

    pub fn as_user(&self) -> Option<&TransactionPartnerUser> {
        match self {
            Self::User(value) => Some(value),
            Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_user(self) -> Option<TransactionPartnerUser> {
        match self {
            Self::User(value) => Some(*value),
            Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_chat(&self) -> Option<&TransactionPartnerChat> {
        match self {
            Self::Chat(value) => Some(value),
            Self::User(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_chat(self) -> Option<TransactionPartnerChat> {
        match self {
            Self::Chat(value) => Some(*value),
            Self::User(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_affiliate_program(&self) -> Option<&TransactionPartnerAffiliateProgram> {
        match self {
            Self::AffiliateProgram(value) => Some(value),
            Self::User(_)
            | Self::Chat(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_affiliate_program(self) -> Option<TransactionPartnerAffiliateProgram> {
        match self {
            Self::AffiliateProgram(value) => Some(*value),
            Self::User(_)
            | Self::Chat(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_fragment(&self) -> Option<&TransactionPartnerFragment> {
        match self {
            Self::Fragment(value) => Some(value),
            Self::User(_)
            | Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_fragment(self) -> Option<TransactionPartnerFragment> {
        match self {
            Self::Fragment(value) => Some(*value),
            Self::User(_)
            | Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_telegram_ads(&self) -> Option<&TransactionPartnerTelegramAds> {
        match self {
            Self::TelegramAds(value) => Some(value),
            Self::User(_)
            | Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramApi(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_telegram_ads(self) -> Option<TransactionPartnerTelegramAds> {
        match self {
            Self::TelegramAds(value) => Some(value),
            Self::User(_)
            | Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramApi(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_telegram_api(&self) -> Option<&TransactionPartnerTelegramApi> {
        match self {
            Self::TelegramApi(value) => Some(value),
            Self::User(_)
            | Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_telegram_api(self) -> Option<TransactionPartnerTelegramApi> {
        match self {
            Self::TelegramApi(value) => Some(*value),
            Self::User(_)
            | Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::Other(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_other(&self) -> Option<&TransactionPartnerOther> {
        match self {
            Self::Other(value) => Some(value),
            Self::User(_)
            | Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn into_other(self) -> Option<TransactionPartnerOther> {
        match self {
            Self::Other(value) => Some(value),
            Self::User(_)
            | Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Unknown(_) => None,
        }
    }

    pub fn as_unknown_value(&self) -> Option<&Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::User(_)
            | Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Other(_) => None,
        }
    }

    pub fn into_unknown_value(self) -> Option<Value> {
        match self {
            Self::Unknown(value) => Some(value),
            Self::User(_)
            | Self::Chat(_)
            | Self::AffiliateProgram(_)
            | Self::Fragment(_)
            | Self::TelegramAds(_)
            | Self::TelegramApi(_)
            | Self::Other(_) => None,
        }
    }
}

impl<'de> Deserialize<'de> for TransactionPartner {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match tagged_kind(&value) {
            Some("user") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::User)
                .map_err(serde::de::Error::custom),
            Some("chat") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::Chat)
                .map_err(serde::de::Error::custom),
            Some("affiliate_program") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::AffiliateProgram)
                .map_err(serde::de::Error::custom),
            Some("fragment") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::Fragment)
                .map_err(serde::de::Error::custom),
            Some("telegram_ads") => serde_json::from_value(strip_type(value))
                .map(Self::TelegramAds)
                .map_err(serde::de::Error::custom),
            Some("telegram_api") => serde_json::from_value(strip_type(value))
                .map(Box::new)
                .map(Self::TelegramApi)
                .map_err(serde::de::Error::custom),
            Some("other") => serde_json::from_value(strip_type(value))
                .map(Self::Other)
                .map_err(serde::de::Error::custom),
            Some(_) | None => Ok(Self::Unknown(value)),
        }
    }
}

/// Telegram Star transaction.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct StarTransaction {
    pub id: String,
    pub amount: i64,
    #[serde(default)]
    pub nanostar_amount: Option<i64>,
    pub date: i64,
    #[serde(default)]
    pub source: Option<TransactionPartner>,
    #[serde(default)]
    pub receiver: Option<TransactionPartner>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram list of Star transactions.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct StarTransactions {
    pub transactions: Vec<StarTransaction>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{
        OwnedGift, RevenueWithdrawalState, TransactionKind, TransactionPartner, UniqueGiftOrigin,
    };

    fn sticker_payload() -> Value {
        json!({
            "file_id": "sticker",
            "file_unique_id": "sticker-unique",
            "type": "regular",
            "width": 64,
            "height": 64,
            "is_animated": false,
            "is_video": false
        })
    }

    fn gift_payload() -> Value {
        json!({
            "id": "gift-1",
            "sticker": sticker_payload(),
            "star_count": 15
        })
    }

    #[test]
    fn owned_gift_regular_parses_tag_and_extra() -> Result<(), Box<dyn std::error::Error>> {
        let gift: OwnedGift = serde_json::from_value(json!({
            "type": "regular",
            "gift": gift_payload(),
            "send_date": 1710000000,
            "is_saved": true,
            "future": "kept"
        }))?;

        let regular = gift.as_regular().ok_or("expected regular owned gift")?;
        assert!(gift.is_regular());
        assert!(!gift.is_unique());
        assert_eq!(gift.kind(), Some("regular"));
        assert_eq!(regular.gift.id, "gift-1");
        assert_eq!(regular.extra["future"], "kept");
        assert_eq!(
            gift.clone()
                .into_regular()
                .ok_or("expected owned regular gift")?
                .gift
                .id,
            "gift-1"
        );
        Ok(())
    }

    #[test]
    fn owned_gift_unknown_variant_is_preserved() -> Result<(), Box<dyn std::error::Error>> {
        let input = json!({
            "type": "future_gift",
            "payload": {"id": "future"}
        });
        let gift: OwnedGift = serde_json::from_value(input.clone())?;

        assert_eq!(gift.kind(), Some("future_gift"));
        assert!(matches!(gift, OwnedGift::Unknown(_)));
        assert_eq!(gift.as_unknown_value(), Some(&input));
        assert_eq!(gift.clone().into_unknown_value(), Some(input.clone()));
        Ok(())
    }

    #[test]
    fn transaction_partner_user_parses_tag_and_extra() -> Result<(), Box<dyn std::error::Error>> {
        let partner: TransactionPartner = serde_json::from_value(json!({
            "type": "user",
            "transaction_type": "gift_purchase",
            "user": {"id": 7, "is_bot": false, "first_name": "Ada"},
            "future": 42
        }))?;

        let user = partner
            .as_user()
            .ok_or("expected user transaction partner")?;
        assert_eq!(partner.kind(), Some("user"));
        assert_eq!(user.transaction_type, TransactionKind::GiftPurchase);
        assert_eq!(user.extra["future"], 42);
        assert_eq!(
            partner
                .clone()
                .into_user()
                .ok_or("expected owned user transaction partner")?
                .user
                .first_name,
            "Ada"
        );
        Ok(())
    }

    #[test]
    fn revenue_withdrawal_pending_preserves_future_fields() -> Result<(), Box<dyn std::error::Error>>
    {
        let state: RevenueWithdrawalState = serde_json::from_value(json!({
            "type": "pending",
            "checkpoint": "queued"
        }))?;

        let pending = state
            .as_pending()
            .ok_or("expected pending withdrawal state")?;
        assert_eq!(state.kind(), Some("pending"));
        assert_eq!(pending.extra["checkpoint"], "queued");
        assert_eq!(
            state
                .clone()
                .into_pending()
                .ok_or("expected owned pending withdrawal state")?
                .extra["checkpoint"],
            "queued"
        );
        Ok(())
    }

    #[test]
    fn string_enums_keep_unknown_values() -> Result<(), Box<dyn std::error::Error>> {
        let origin: UniqueGiftOrigin = serde_json::from_value(json!("future_origin"))?;
        let transaction: TransactionKind = serde_json::from_value(json!("future_transaction"))?;

        assert_eq!(origin.as_str(), "future_origin");
        assert_eq!(transaction.as_str(), "future_transaction");
        Ok(())
    }
}
