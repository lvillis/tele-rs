use std::collections::BTreeMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::extra::{
    field_len as extra_field_len, serialize_fields as serialize_extra_fields,
    serialize_optional_field,
};

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

impl Serialize for Invoice {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "title",
            "description",
            "start_parameter",
            "currency",
            "total_amount",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 5))?;
        object.serialize_entry("title", &self.title)?;
        object.serialize_entry("description", &self.description)?;
        object.serialize_entry("start_parameter", &self.start_parameter)?;
        object.serialize_entry("currency", &self.currency)?;
        object.serialize_entry("total_amount", &self.total_amount)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
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

impl Serialize for ShippingAddress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "country_code",
            "state",
            "city",
            "street_line1",
            "street_line2",
            "post_code",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let mut object = serializer.serialize_map(Some(extra_len + 6))?;
        object.serialize_entry("country_code", &self.country_code)?;
        object.serialize_entry("state", &self.state)?;
        object.serialize_entry("city", &self.city)?;
        object.serialize_entry("street_line1", &self.street_line1)?;
        object.serialize_entry("street_line2", &self.street_line2)?;
        object.serialize_entry("post_code", &self.post_code)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram order info payload.
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for OrderInfo {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["name", "phone_number", "email", "shipping_address"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.name.is_some())
            + usize::from(self.phone_number.is_some())
            + usize::from(self.email.is_some())
            + usize::from(self.shipping_address.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len))?;
        serialize_optional_field(&mut object, "name", &self.name)?;
        serialize_optional_field(&mut object, "phone_number", &self.phone_number)?;
        serialize_optional_field(&mut object, "email", &self.email)?;
        serialize_optional_field(&mut object, "shipping_address", &self.shipping_address)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram successful payment payload.
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for SuccessfulPayment {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "currency",
            "total_amount",
            "invoice_payload",
            "subscription_expiration_date",
            "is_recurring",
            "is_first_recurring",
            "shipping_option_id",
            "order_info",
            "telegram_payment_charge_id",
            "provider_payment_charge_id",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.subscription_expiration_date.is_some())
            + usize::from(self.is_recurring)
            + usize::from(self.is_first_recurring)
            + usize::from(self.shipping_option_id.is_some())
            + usize::from(self.order_info.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 5))?;
        object.serialize_entry("currency", &self.currency)?;
        object.serialize_entry("total_amount", &self.total_amount)?;
        object.serialize_entry("invoice_payload", &self.invoice_payload)?;
        serialize_optional_field(
            &mut object,
            "subscription_expiration_date",
            &self.subscription_expiration_date,
        )?;
        if self.is_recurring {
            object.serialize_entry("is_recurring", &self.is_recurring)?;
        }
        if self.is_first_recurring {
            object.serialize_entry("is_first_recurring", &self.is_first_recurring)?;
        }
        serialize_optional_field(&mut object, "shipping_option_id", &self.shipping_option_id)?;
        serialize_optional_field(&mut object, "order_info", &self.order_info)?;
        object.serialize_entry(
            "telegram_payment_charge_id",
            &self.telegram_payment_charge_id,
        )?;
        object.serialize_entry(
            "provider_payment_charge_id",
            &self.provider_payment_charge_id,
        )?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram refunded payment payload.
#[derive(Clone, Debug, Deserialize)]
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

impl Serialize for RefundedPayment {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = [
            "currency",
            "total_amount",
            "invoice_payload",
            "telegram_payment_charge_id",
            "provider_payment_charge_id",
        ];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.provider_payment_charge_id.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 4))?;
        object.serialize_entry("currency", &self.currency)?;
        object.serialize_entry("total_amount", &self.total_amount)?;
        object.serialize_entry("invoice_payload", &self.invoice_payload)?;
        object.serialize_entry(
            "telegram_payment_charge_id",
            &self.telegram_payment_charge_id,
        )?;
        serialize_optional_field(
            &mut object,
            "provider_payment_charge_id",
            &self.provider_payment_charge_id,
        )?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

/// Telegram amount of Stars.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct StarAmount {
    pub amount: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nanostar_amount: Option<i64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Serialize for StarAmount {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let reserved = ["amount", "nanostar_amount"];
        let extra_len = extra_field_len(&self.extra, &reserved);
        let optional_len = usize::from(self.nanostar_amount.is_some());
        let mut object = serializer.serialize_map(Some(extra_len + optional_len + 1))?;
        object.serialize_entry("amount", &self.amount)?;
        serialize_optional_field(&mut object, "nanostar_amount", &self.nanostar_amount)?;
        serialize_extra_fields(&mut object, &self.extra, &reserved)?;
        object.end()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn invoice_and_shipping_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut invoice: Invoice = serde_json::from_value(json!({
            "title": "Invoice",
            "description": "Description",
            "start_parameter": "start",
            "currency": "USD",
            "total_amount": 1000,
            "future": {"kept": true}
        }))?;
        invoice.extra.insert("title".to_owned(), json!("spoofed"));
        invoice
            .extra
            .insert("description".to_owned(), json!("spoofed"));
        invoice.extra.insert("currency".to_owned(), json!("EUR"));
        invoice.extra.insert("total_amount".to_owned(), json!(1));
        invoice
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let invoice_value = serde_json::to_value(invoice)?;
        assert_eq!(invoice_value["title"], "Invoice");
        assert_eq!(invoice_value["description"], "Description");
        assert_eq!(invoice_value["currency"], "USD");
        assert_eq!(invoice_value["total_amount"], 1000);
        assert_eq!(invoice_value["future"], json!({"kept": true}));
        assert_eq!(invoice_value["another_future"], "kept");

        let mut address: ShippingAddress = serde_json::from_value(json!({
            "country_code": "US",
            "state": "CA",
            "city": "San Francisco",
            "street_line1": "1 Market",
            "street_line2": "Suite 1",
            "post_code": "94105",
            "future": {"kept": true}
        }))?;
        address.extra.insert("country_code".to_owned(), json!("CN"));
        address.extra.insert("city".to_owned(), json!("spoofed"));
        address.extra.insert("post_code".to_owned(), json!("00000"));
        address
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let address_value = serde_json::to_value(address)?;
        assert_eq!(address_value["country_code"], "US");
        assert_eq!(address_value["city"], "San Francisco");
        assert_eq!(address_value["post_code"], "94105");
        assert_eq!(address_value["future"], json!({"kept": true}));
        assert_eq!(address_value["another_future"], "kept");
        Ok(())
    }

    #[test]
    fn order_and_payment_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut order: OrderInfo = serde_json::from_value(json!({
            "name": "Alice",
            "email": "alice@example.com",
            "future": {"kept": true}
        }))?;
        order.extra.insert("name".to_owned(), json!("spoofed"));
        order
            .extra
            .insert("phone_number".to_owned(), json!("spoofed"));
        order
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let order_value = serde_json::to_value(order)?;
        assert_eq!(order_value["name"], "Alice");
        assert_eq!(order_value["email"], "alice@example.com");
        assert!(order_value.get("phone_number").is_none());
        assert_eq!(order_value["future"], json!({"kept": true}));
        assert_eq!(order_value["another_future"], "kept");

        let mut payment: SuccessfulPayment = serde_json::from_value(json!({
            "currency": "USD",
            "total_amount": 1000,
            "invoice_payload": "payload",
            "telegram_payment_charge_id": "telegram-charge",
            "provider_payment_charge_id": "provider-charge",
            "future": {"kept": true}
        }))?;
        payment.extra.insert("currency".to_owned(), json!("EUR"));
        payment.extra.insert("total_amount".to_owned(), json!(1));
        payment
            .extra
            .insert("invoice_payload".to_owned(), json!("spoofed"));
        payment.extra.insert("is_recurring".to_owned(), json!(true));
        payment
            .extra
            .insert("telegram_payment_charge_id".to_owned(), json!("spoofed"));
        payment
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let payment_value = serde_json::to_value(payment)?;
        assert_eq!(payment_value["currency"], "USD");
        assert_eq!(payment_value["total_amount"], 1000);
        assert_eq!(payment_value["invoice_payload"], "payload");
        assert_eq!(
            payment_value["telegram_payment_charge_id"],
            "telegram-charge"
        );
        assert!(payment_value.get("is_recurring").is_none());
        assert_eq!(payment_value["future"], json!({"kept": true}));
        assert_eq!(payment_value["another_future"], "kept");
        Ok(())
    }

    #[test]
    fn refund_and_star_amount_extra_cannot_override_reserved_fields()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut refund: RefundedPayment = serde_json::from_value(json!({
            "currency": "USD",
            "total_amount": 1000,
            "invoice_payload": "payload",
            "telegram_payment_charge_id": "telegram-charge",
            "future": {"kept": true}
        }))?;
        refund.extra.insert("currency".to_owned(), json!("EUR"));
        refund.extra.insert("total_amount".to_owned(), json!(1));
        refund.extra.insert(
            "provider_payment_charge_id".to_owned(),
            json!("provider-charge"),
        );
        refund
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let refund_value = serde_json::to_value(refund)?;
        assert_eq!(refund_value["currency"], "USD");
        assert_eq!(refund_value["total_amount"], 1000);
        assert_eq!(refund_value["invoice_payload"], "payload");
        assert_eq!(
            refund_value["telegram_payment_charge_id"],
            "telegram-charge"
        );
        assert!(refund_value.get("provider_payment_charge_id").is_none());
        assert_eq!(refund_value["future"], json!({"kept": true}));
        assert_eq!(refund_value["another_future"], "kept");

        let mut amount: StarAmount = serde_json::from_value(json!({
            "amount": 10,
            "future": {"kept": true}
        }))?;
        amount.extra.insert("amount".to_owned(), json!(1));
        amount.extra.insert("nanostar_amount".to_owned(), json!(1));
        amount
            .extra
            .insert("another_future".to_owned(), json!("kept"));

        let amount_value = serde_json::to_value(amount)?;
        assert_eq!(amount_value["amount"], 10);
        assert!(amount_value.get("nanostar_amount").is_none());
        assert_eq!(amount_value["future"], json!({"kept": true}));
        assert_eq!(amount_value["another_future"], "kept");
        Ok(())
    }
}
