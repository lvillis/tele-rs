use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::is_false;

/// Telegram invoice payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Invoice {
    pub title: String,
    pub description: String,
    pub start_parameter: String,
    pub currency: String,
    pub total_amount: i64,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram shipping address payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ShippingAddress {
    pub country_code: String,
    pub state: String,
    pub city: String,
    pub street_line1: String,
    pub street_line2: String,
    pub post_code: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram order info payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct OrderInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<ShippingAddress>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram successful payment payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SuccessfulPayment {
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_expiration_date: Option<i64>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_recurring: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_first_recurring: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shipping_option_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_info: Option<OrderInfo>,
    pub telegram_payment_charge_id: String,
    pub provider_payment_charge_id: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram refunded payment payload.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RefundedPayment {
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    pub telegram_payment_charge_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_payment_charge_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram amount of Stars.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StarAmount {
    pub amount: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nanostar_amount: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
