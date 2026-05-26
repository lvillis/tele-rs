use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

/// Telegram invoice payload.
#[derive(Clone, Debug, Deserialize)]
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
#[derive(Clone, Debug, Deserialize)]
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
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct OrderInfo {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub phone_number: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub shipping_address: Option<ShippingAddress>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram successful payment payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct SuccessfulPayment {
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    #[serde(default)]
    pub subscription_expiration_date: Option<i64>,
    #[serde(default)]
    pub is_recurring: bool,
    #[serde(default)]
    pub is_first_recurring: bool,
    #[serde(default)]
    pub shipping_option_id: Option<String>,
    #[serde(default)]
    pub order_info: Option<OrderInfo>,
    pub telegram_payment_charge_id: String,
    pub provider_payment_charge_id: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram refunded payment payload.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct RefundedPayment {
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    pub telegram_payment_charge_id: String,
    #[serde(default)]
    pub provider_payment_charge_id: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Telegram amount of Stars.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct StarAmount {
    pub amount: i64,
    #[serde(default)]
    pub nanostar_amount: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn invoice_and_shipping_preserve_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let invoice: Invoice = serde_json::from_value(json!({
            "title": "Invoice",
            "description": "Description",
            "start_parameter": "start",
            "currency": "USD",
            "total_amount": 1000,
            "future": {"kept": true}
        }))?;
        assert_eq!(invoice.title, "Invoice");
        assert_eq!(invoice.currency, "USD");
        assert_eq!(invoice.total_amount, 1000);
        assert_eq!(invoice.extra.get("future"), Some(&json!({"kept": true})));

        let address: ShippingAddress = serde_json::from_value(json!({
            "country_code": "US",
            "state": "CA",
            "city": "San Francisco",
            "street_line1": "1 Market",
            "street_line2": "Suite 1",
            "post_code": "94105",
            "future": {"kept": true}
        }))?;
        assert_eq!(address.country_code, "US");
        assert_eq!(address.city, "San Francisco");
        assert_eq!(address.post_code, "94105");
        assert_eq!(address.extra.get("future"), Some(&json!({"kept": true})));
        Ok(())
    }

    #[test]
    fn order_and_payment_preserve_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let order: OrderInfo = serde_json::from_value(json!({
            "name": "Alice",
            "email": "alice@example.com",
            "future": {"kept": true}
        }))?;
        assert_eq!(order.name.as_deref(), Some("Alice"));
        assert_eq!(order.email.as_deref(), Some("alice@example.com"));
        assert_eq!(order.extra.get("future"), Some(&json!({"kept": true})));

        let payment: SuccessfulPayment = serde_json::from_value(json!({
            "currency": "USD",
            "total_amount": 1000,
            "invoice_payload": "payload",
            "telegram_payment_charge_id": "telegram-charge",
            "provider_payment_charge_id": "provider-charge",
            "future": {"kept": true}
        }))?;
        assert_eq!(payment.currency, "USD");
        assert_eq!(payment.total_amount, 1000);
        assert_eq!(payment.invoice_payload, "payload");
        assert_eq!(payment.telegram_payment_charge_id, "telegram-charge");
        assert_eq!(payment.extra.get("future"), Some(&json!({"kept": true})));
        Ok(())
    }

    #[test]
    fn refund_and_star_amount_preserve_future_fields() -> Result<(), Box<dyn std::error::Error>> {
        let refund: RefundedPayment = serde_json::from_value(json!({
            "currency": "USD",
            "total_amount": 1000,
            "invoice_payload": "payload",
            "telegram_payment_charge_id": "telegram-charge",
            "future": {"kept": true}
        }))?;
        assert_eq!(refund.currency, "USD");
        assert_eq!(refund.total_amount, 1000);
        assert_eq!(refund.invoice_payload, "payload");
        assert_eq!(refund.telegram_payment_charge_id, "telegram-charge");
        assert_eq!(refund.extra.get("future"), Some(&json!({"kept": true})));

        let amount: StarAmount = serde_json::from_value(json!({
            "amount": 10,
            "future": {"kept": true}
        }))?;
        assert_eq!(amount.amount, 10);
        assert_eq!(amount.extra.get("future"), Some(&json!({"kept": true})));
        Ok(())
    }
}
