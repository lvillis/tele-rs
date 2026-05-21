use serde::{Deserialize, Serialize};

use crate::Error;
use crate::types::common::ChatId;
use crate::types::telegram::{ReplyMarkup, ReplyParameters, SuggestedPostParameters};
use crate::types::validation::{
    optional_request_positive_i64 as validate_optional_positive_i64,
    optional_request_positive_u32 as validate_optional_positive_u32,
    optional_request_string_id as validate_optional_id, request_non_empty as ensure_non_empty,
    request_string_id as validate_id,
    suggested_post_parameters as validate_suggested_post_parameters,
};

const INVOICE_TITLE_MAX_CHARS: usize = 32;
const INVOICE_DESCRIPTION_MAX_CHARS: usize = 255;
const INVOICE_PAYLOAD_MAX_BYTES: usize = 128;
const TELEGRAM_STARS_SUBSCRIPTION_PERIOD_SECS: u32 = 2_592_000;

fn validate_bounded_text(
    method: &str,
    field: &str,
    value: &str,
    max_chars: usize,
) -> Result<(), Error> {
    ensure_non_empty(method, field, value)?;
    if value.chars().count() > max_chars {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires `{field}` to be at most {max_chars} characters"),
        });
    }

    Ok(())
}

fn validate_payload(method: &str, payload: &str) -> Result<(), Error> {
    ensure_non_empty(method, "payload", payload)?;
    if payload.len() > INVOICE_PAYLOAD_MAX_BYTES {
        return Err(Error::InvalidRequest {
            reason: format!(
                "{method} requires `payload` to be at most {INVOICE_PAYLOAD_MAX_BYTES} bytes"
            ),
        });
    }

    Ok(())
}

fn validate_currency(method: &str, currency: &str) -> Result<(), Error> {
    ensure_non_empty(method, "currency", currency)?;

    let is_valid = currency.len() == 3 && currency.bytes().all(|byte| byte.is_ascii_uppercase());
    if !is_valid {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires a 3-letter uppercase `currency` code"),
        });
    }

    Ok(())
}

fn validate_prices(method: &str, prices: &[LabeledPrice]) -> Result<(), Error> {
    if prices.is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires at least one price item"),
        });
    }

    for (index, price) in prices.iter().enumerate() {
        if price.label.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: format!("{method} price at index {index} requires non-empty `label`"),
            });
        }
    }

    Ok(())
}

fn validate_shipping_options(
    method: &str,
    shipping_options: &[ShippingOption],
) -> Result<(), Error> {
    if shipping_options.is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires at least one shipping option when ok=true"),
        });
    }

    for (index, option) in shipping_options.iter().enumerate() {
        validate_id(method, &format!("shipping_options[{index}].id"), &option.id)?;
        ensure_non_empty(
            method,
            &format!("shipping_options[{index}].title"),
            &option.title,
        )?;
        validate_prices(method, &option.prices)?;
    }

    Ok(())
}

fn validate_tip_configuration(
    method: &str,
    max_tip_amount: Option<i64>,
    suggested_tip_amounts: Option<&[i64]>,
) -> Result<(), Error> {
    if let Some(max_tip_amount) = max_tip_amount
        && max_tip_amount <= 0
    {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires `max_tip_amount` to be greater than zero"),
        });
    }

    let Some(suggested_tip_amounts) = suggested_tip_amounts else {
        return Ok(());
    };

    if suggested_tip_amounts.is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires non-empty `suggested_tip_amounts` when provided"),
        });
    }

    if suggested_tip_amounts.len() > 4 {
        return Err(Error::InvalidRequest {
            reason: format!("{method} supports at most 4 `suggested_tip_amounts` entries"),
        });
    }

    let Some(max_tip_amount) = max_tip_amount else {
        return Err(Error::InvalidRequest {
            reason: format!(
                "{method} requires `max_tip_amount` when using `suggested_tip_amounts`"
            ),
        });
    };

    let mut previous = 0_i64;
    for amount in suggested_tip_amounts {
        if *amount <= 0 {
            return Err(Error::InvalidRequest {
                reason: format!("{method} requires positive values in `suggested_tip_amounts`"),
            });
        }
        if *amount > max_tip_amount {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "{method} requires each `suggested_tip_amounts` value to be <= `max_tip_amount`"
                ),
            });
        }
        if *amount <= previous {
            return Err(Error::InvalidRequest {
                reason: format!(
                    "{method} requires strictly increasing values in `suggested_tip_amounts`"
                ),
            });
        }
        previous = *amount;
    }

    Ok(())
}

fn validate_subscription_period(method: &str, value: Option<u32>) -> Result<(), Error> {
    if let Some(value) = value
        && value != TELEGRAM_STARS_SUBSCRIPTION_PERIOD_SECS
    {
        return Err(Error::InvalidRequest {
            reason: format!(
                "{method} requires `subscription_period` to be {TELEGRAM_STARS_SUBSCRIPTION_PERIOD_SECS} seconds when provided"
            ),
        });
    }

    Ok(())
}

/// Telegram invoice price item.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LabeledPrice {
    pub label: String,
    pub amount: i64,
}

impl LabeledPrice {
    pub fn new(label: impl Into<String>, amount: i64) -> Self {
        Self {
            label: label.into(),
            amount,
        }
    }
}

/// Telegram shipping option.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShippingOption {
    pub id: String,
    pub title: String,
    pub prices: Vec<LabeledPrice>,
}

impl ShippingOption {
    pub fn new(id: impl Into<String>, title: impl Into<String>, prices: Vec<LabeledPrice>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            prices,
        }
    }
}

/// `sendInvoice` request.
#[derive(Clone, Debug, Serialize)]
pub struct SendInvoiceRequest {
    pub chat_id: ChatId,
    pub title: String,
    pub description: String,
    pub payload: String,
    pub currency: String,
    pub prices: Vec<LabeledPrice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_thread_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_messages_topic_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_token: Option<String>,
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
    pub photo_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_height: Option<u32>,
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
    pub suggested_post_parameters: Option<SuggestedPostParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_parameters: Option<ReplyParameters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<ReplyMarkup>,
}

impl SendInvoiceRequest {
    pub fn new(
        chat_id: impl Into<ChatId>,
        title: impl Into<String>,
        description: impl Into<String>,
        payload: impl Into<String>,
        currency: impl Into<String>,
        prices: Vec<LabeledPrice>,
    ) -> Result<Self, Error> {
        let title = title.into();
        let description = description.into();
        let payload = payload.into();
        let currency = currency.into();

        validate_bounded_text("sendInvoice", "title", &title, INVOICE_TITLE_MAX_CHARS)?;
        validate_bounded_text(
            "sendInvoice",
            "description",
            &description,
            INVOICE_DESCRIPTION_MAX_CHARS,
        )?;
        validate_payload("sendInvoice", &payload)?;
        validate_currency("sendInvoice", &currency)?;
        validate_prices("sendInvoice", &prices)?;

        let request = Self {
            chat_id: chat_id.into(),
            title,
            description,
            payload,
            currency,
            prices,
            message_thread_id: None,
            direct_messages_topic_id: None,
            provider_token: None,
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
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.chat_id.validate()?;
        validate_optional_positive_i64("sendInvoice", "message_thread_id", self.message_thread_id)?;
        validate_optional_positive_i64(
            "sendInvoice",
            "direct_messages_topic_id",
            self.direct_messages_topic_id,
        )?;
        if let Some(reply_parameters) = self.reply_parameters.as_ref() {
            reply_parameters.validate()?;
        }
        if let Some(reply_markup) = self.reply_markup.as_ref() {
            reply_markup.validate()?;
        }
        validate_bounded_text("sendInvoice", "title", &self.title, INVOICE_TITLE_MAX_CHARS)?;
        validate_bounded_text(
            "sendInvoice",
            "description",
            &self.description,
            INVOICE_DESCRIPTION_MAX_CHARS,
        )?;
        validate_payload("sendInvoice", &self.payload)?;
        validate_currency("sendInvoice", &self.currency)?;
        validate_prices("sendInvoice", &self.prices)?;
        validate_tip_configuration(
            "sendInvoice",
            self.max_tip_amount,
            self.suggested_tip_amounts.as_deref(),
        )?;
        validate_optional_positive_u32("sendInvoice", "photo_size", self.photo_size)?;
        validate_optional_positive_u32("sendInvoice", "photo_width", self.photo_width)?;
        validate_optional_positive_u32("sendInvoice", "photo_height", self.photo_height)?;
        validate_optional_id(
            "sendInvoice",
            "message_effect_id",
            self.message_effect_id.as_deref(),
        )?;
        validate_suggested_post_parameters(self.suggested_post_parameters.as_ref())?;
        Ok(())
    }

    pub fn suggested_post_parameters(
        mut self,
        suggested_post_parameters: SuggestedPostParameters,
    ) -> Self {
        self.suggested_post_parameters = Some(suggested_post_parameters);
        self
    }

    pub fn reply_parameters(mut self, reply_parameters: ReplyParameters) -> Self {
        self.reply_parameters = Some(reply_parameters);
        self
    }

    pub fn reply_markup(mut self, reply_markup: impl Into<ReplyMarkup>) -> Self {
        self.reply_markup = Some(reply_markup.into());
        self
    }
}

/// `createInvoiceLink` request.
#[derive(Clone, Debug, Serialize)]
pub struct CreateInvoiceLinkRequest {
    pub title: String,
    pub description: String,
    pub payload: String,
    pub currency: String,
    pub prices: Vec<LabeledPrice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub business_connection_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_period: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tip_amount: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_tip_amounts: Option<Vec<i64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photo_height: Option<u32>,
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

impl CreateInvoiceLinkRequest {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        payload: impl Into<String>,
        currency: impl Into<String>,
        prices: Vec<LabeledPrice>,
    ) -> Result<Self, Error> {
        let title = title.into();
        let description = description.into();
        let payload = payload.into();
        let currency = currency.into();

        validate_bounded_text(
            "createInvoiceLink",
            "title",
            &title,
            INVOICE_TITLE_MAX_CHARS,
        )?;
        validate_bounded_text(
            "createInvoiceLink",
            "description",
            &description,
            INVOICE_DESCRIPTION_MAX_CHARS,
        )?;
        validate_payload("createInvoiceLink", &payload)?;
        validate_currency("createInvoiceLink", &currency)?;
        validate_prices("createInvoiceLink", &prices)?;

        let request = Self {
            title,
            description,
            payload,
            currency,
            prices,
            business_connection_id: None,
            subscription_period: None,
            provider_token: None,
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
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_optional_id(
            "createInvoiceLink",
            "business_connection_id",
            self.business_connection_id.as_deref(),
        )?;
        validate_bounded_text(
            "createInvoiceLink",
            "title",
            &self.title,
            INVOICE_TITLE_MAX_CHARS,
        )?;
        validate_bounded_text(
            "createInvoiceLink",
            "description",
            &self.description,
            INVOICE_DESCRIPTION_MAX_CHARS,
        )?;
        validate_payload("createInvoiceLink", &self.payload)?;
        validate_currency("createInvoiceLink", &self.currency)?;
        validate_prices("createInvoiceLink", &self.prices)?;
        validate_tip_configuration(
            "createInvoiceLink",
            self.max_tip_amount,
            self.suggested_tip_amounts.as_deref(),
        )?;
        validate_subscription_period("createInvoiceLink", self.subscription_period)?;
        validate_optional_positive_u32("createInvoiceLink", "photo_size", self.photo_size)?;
        validate_optional_positive_u32("createInvoiceLink", "photo_width", self.photo_width)?;
        validate_optional_positive_u32("createInvoiceLink", "photo_height", self.photo_height)?;
        Ok(())
    }
}

/// `answerShippingQuery` request.
#[derive(Clone, Debug, Serialize)]
pub struct AnswerShippingQueryRequest {
    pub shipping_query_id: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shipping_options: Option<Vec<ShippingOption>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl AnswerShippingQueryRequest {
    pub fn new(shipping_query_id: impl Into<String>, ok: bool) -> Self {
        Self {
            shipping_query_id: shipping_query_id.into(),
            ok,
            shipping_options: None,
            error_message: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_id(
            "answerShippingQuery",
            "shipping_query_id",
            &self.shipping_query_id,
        )?;

        if self.ok {
            if self.error_message.is_some() {
                return Err(Error::InvalidRequest {
                    reason: "answerShippingQuery must omit error_message when ok=true".to_owned(),
                });
            }
            let Some(shipping_options) = self.shipping_options.as_deref() else {
                return Err(Error::InvalidRequest {
                    reason: "answerShippingQuery requires shipping_options when ok=true".to_owned(),
                });
            };
            validate_shipping_options("answerShippingQuery", shipping_options)?;
            return Ok(());
        }

        if self.shipping_options.is_some() {
            return Err(Error::InvalidRequest {
                reason: "answerShippingQuery must omit shipping_options when ok=false".to_owned(),
            });
        }
        if self
            .error_message
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(Error::InvalidRequest {
                reason: "answerShippingQuery requires non-empty error_message when ok=false"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

/// `answerPreCheckoutQuery` request.
#[derive(Clone, Debug, Serialize)]
pub struct AnswerPreCheckoutQueryRequest {
    pub pre_checkout_query_id: String,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl AnswerPreCheckoutQueryRequest {
    pub fn new(pre_checkout_query_id: impl Into<String>, ok: bool) -> Self {
        Self {
            pre_checkout_query_id: pre_checkout_query_id.into(),
            ok,
            error_message: None,
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        validate_id(
            "answerPreCheckoutQuery",
            "pre_checkout_query_id",
            &self.pre_checkout_query_id,
        )?;

        if self.ok {
            if self.error_message.is_some() {
                return Err(Error::InvalidRequest {
                    reason: "answerPreCheckoutQuery must omit error_message when ok=true"
                        .to_owned(),
                });
            }
            return Ok(());
        }

        if self
            .error_message
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(Error::InvalidRequest {
                reason: "answerPreCheckoutQuery requires non-empty error_message when ok=false"
                    .to_owned(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_payment_request_ids_and_targets() -> Result<(), Error> {
        let invoice = SendInvoiceRequest::new(
            0_i64,
            "title",
            "description",
            "payload",
            "USD",
            vec![LabeledPrice::new("item", 100)],
        );
        assert!(matches!(invoice, Err(Error::InvalidRequest { .. })));

        let invoice = SendInvoiceRequest::new(
            1_i64,
            "t".repeat(INVOICE_TITLE_MAX_CHARS + 1),
            "description",
            "payload",
            "USD",
            vec![LabeledPrice::new("item", 100)],
        );
        assert!(matches!(invoice, Err(Error::InvalidRequest { .. })));

        let invoice = SendInvoiceRequest::new(
            1_i64,
            "title",
            "description",
            "x".repeat(INVOICE_PAYLOAD_MAX_BYTES + 1),
            "USD",
            vec![LabeledPrice::new("item", 100)],
        );
        assert!(matches!(invoice, Err(Error::InvalidRequest { .. })));

        let mut invoice = SendInvoiceRequest::new(
            1_i64,
            "title",
            "description",
            "payload",
            "USD",
            vec![LabeledPrice::new("item", 100)],
        )?;
        invoice.message_thread_id = Some(0);
        assert!(matches!(
            invoice.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invoice.message_thread_id = None;
        invoice.direct_messages_topic_id = Some(-1);
        assert!(matches!(
            invoice.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invoice.direct_messages_topic_id = None;
        invoice.photo_width = Some(0);
        assert!(matches!(
            invoice.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invoice.photo_width = None;
        invoice.reply_markup =
            Some(crate::types::telegram::InlineKeyboardMarkup::new(Vec::new()).into());
        assert!(matches!(
            invoice.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invoice.reply_markup = None;
        invoice.message_effect_id = Some("bad\nid".to_owned());
        assert!(matches!(
            invoice.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invoice.message_effect_id = None;
        invoice.suggested_post_parameters =
            Some(SuggestedPostParameters::new(serde_json::json!({
                "send_date": 1
            }))?);
        invoice.validate()?;
        let serialized =
            serde_json::to_value(&invoice).map_err(|source| Error::SerializeRequest { source })?;
        assert!(serialized.get("suggested_post_parameters").is_some());

        let mut invoice_link = CreateInvoiceLinkRequest::new(
            "title",
            "description",
            "payload",
            "USD",
            vec![LabeledPrice::new("item", 100)],
        )?;
        invoice_link.business_connection_id = Some(" \n ".to_owned());
        assert!(matches!(
            invoice_link.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invoice_link.business_connection_id = None;
        invoice_link.subscription_period = Some(0);
        assert!(matches!(
            invoice_link.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invoice_link.subscription_period = Some(TELEGRAM_STARS_SUBSCRIPTION_PERIOD_SECS);
        invoice_link.validate()?;

        let mut shipping = AnswerShippingQueryRequest::new("", true);
        shipping.shipping_options = Some(vec![ShippingOption::new(
            "standard",
            "Standard",
            vec![LabeledPrice::new("shipping", 100)],
        )]);
        assert!(matches!(
            shipping.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let shipping = AnswerShippingQueryRequest::new("ship-1", true);
        assert!(matches!(
            shipping.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut shipping = AnswerShippingQueryRequest::new("ship-1", true);
        shipping.shipping_options = Some(vec![ShippingOption::new(
            "standard",
            "Standard",
            vec![LabeledPrice::new("shipping", 100)],
        )]);
        shipping.error_message = Some("unavailable".to_owned());
        assert!(matches!(
            shipping.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut shipping = AnswerShippingQueryRequest::new("ship-1", false);
        shipping.shipping_options = Some(vec![ShippingOption::new(
            "standard",
            "Standard",
            vec![LabeledPrice::new("shipping", 100)],
        )]);
        shipping.error_message = Some("unavailable".to_owned());
        assert!(matches!(
            shipping.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let checkout = AnswerPreCheckoutQueryRequest::new("bad\nid", true);
        assert!(matches!(
            checkout.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut checkout = AnswerPreCheckoutQueryRequest::new("checkout-1", true);
        checkout.error_message = Some("declined".to_owned());
        assert!(matches!(
            checkout.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        Ok(())
    }
}
