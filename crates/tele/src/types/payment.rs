use serde::{Deserialize, Serialize};

use crate::Error;
use crate::types::common::ChatId;
use crate::types::telegram::{InlineKeyboardMarkup, ReplyParameters, SuggestedPostParameters};
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
const TELEGRAM_STARS_CURRENCY: &str = "XTR";
const TELEGRAM_STARS_SUBSCRIPTION_PERIOD_SECS: u32 = 2_592_000;
const TELEGRAM_STARS_SUBSCRIPTION_PRICE_MAX: i64 = 10_000;

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

pub(crate) fn validate_invoice_title(method: &str, title: &str) -> Result<(), Error> {
    validate_bounded_text(method, "title", title, INVOICE_TITLE_MAX_CHARS)
}

pub(crate) fn validate_invoice_description(method: &str, description: &str) -> Result<(), Error> {
    validate_bounded_text(
        method,
        "description",
        description,
        INVOICE_DESCRIPTION_MAX_CHARS,
    )
}

pub(crate) fn validate_invoice_payload(method: &str, payload: &str) -> Result<(), Error> {
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

pub(crate) fn validate_invoice_currency(method: &str, currency: &str) -> Result<(), Error> {
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
        price.validate_with_context(method, index)?;
    }

    Ok(())
}

pub(crate) fn validate_invoice_prices(
    method: &str,
    currency: &str,
    prices: &[LabeledPrice],
) -> Result<(), Error> {
    validate_prices(method, prices)?;

    if currency == TELEGRAM_STARS_CURRENCY && prices.len() != 1 {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires exactly one price item for Telegram Stars payments"),
        });
    }

    Ok(())
}

pub(crate) fn validate_invoice_business_connection_id(
    method: &str,
    currency: &str,
    value: Option<&str>,
) -> Result<(), Error> {
    validate_optional_id(method, "business_connection_id", value)?;

    if value.is_some() && currency != TELEGRAM_STARS_CURRENCY {
        return Err(Error::InvalidRequest {
            reason: format!(
                "{method} supports `business_connection_id` only for Telegram Stars payments"
            ),
        });
    }

    Ok(())
}

pub(crate) fn validate_invoice_reply_markup(
    method: &str,
    reply_markup: Option<&InlineKeyboardMarkup>,
) -> Result<(), Error> {
    let Some(reply_markup) = reply_markup else {
        return Ok(());
    };

    reply_markup.validate()?;
    let Some(first_button) = reply_markup
        .inline_keyboard
        .first()
        .and_then(|row| row.first())
    else {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires a non-empty inline keyboard"),
        });
    };

    if !first_button.is_pay_button() {
        return Err(Error::InvalidRequest {
            reason: format!(
                "{method} requires the first inline keyboard button to be a Pay button"
            ),
        });
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

pub(crate) fn validate_invoice_tip_configuration(
    method: &str,
    currency: &str,
    max_tip_amount: Option<i64>,
    suggested_tip_amounts: Option<&[i64]>,
) -> Result<(), Error> {
    if currency == TELEGRAM_STARS_CURRENCY
        && (max_tip_amount.is_some() || suggested_tip_amounts.is_some())
    {
        return Err(Error::InvalidRequest {
            reason: format!("{method} does not support tips for Telegram Stars payments"),
        });
    }

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

pub(crate) fn validate_invoice_subscription_period(
    method: &str,
    currency: &str,
    value: Option<i64>,
    prices: &[LabeledPrice],
) -> Result<(), Error> {
    let Some(value) = value else {
        return Ok(());
    };

    if currency != TELEGRAM_STARS_CURRENCY {
        return Err(Error::InvalidRequest {
            reason: format!(
                "{method} requires `currency` to be {TELEGRAM_STARS_CURRENCY} when using `subscription_period`"
            ),
        });
    }

    if value != i64::from(TELEGRAM_STARS_SUBSCRIPTION_PERIOD_SECS) {
        return Err(Error::InvalidRequest {
            reason: format!(
                "{method} requires `subscription_period` to be {TELEGRAM_STARS_SUBSCRIPTION_PERIOD_SECS} seconds when provided"
            ),
        });
    }

    if let Some(price) = prices.first()
        && price.amount > TELEGRAM_STARS_SUBSCRIPTION_PRICE_MAX
    {
        return Err(Error::InvalidRequest {
            reason: format!(
                "{method} requires subscription price to be at most {TELEGRAM_STARS_SUBSCRIPTION_PRICE_MAX} Telegram Stars"
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

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("labeledPrice", "label", &self.label)
    }

    fn validate_with_context(&self, method: &str, index: usize) -> Result<(), Error> {
        if self.label.trim().is_empty() {
            return Err(Error::InvalidRequest {
                reason: format!("{method} price at index {index} requires non-empty `label`"),
            });
        }

        Ok(())
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

    pub fn validate(&self) -> Result<(), Error> {
        validate_id("shippingOption", "id", &self.id)?;
        ensure_non_empty("shippingOption", "title", &self.title)?;
        validate_prices("shippingOption", &self.prices)
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
    pub reply_markup: Option<InlineKeyboardMarkup>,
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

        validate_invoice_title("sendInvoice", &title)?;
        validate_invoice_description("sendInvoice", &description)?;
        validate_invoice_payload("sendInvoice", &payload)?;
        validate_invoice_currency("sendInvoice", &currency)?;
        validate_invoice_prices("sendInvoice", &currency, &prices)?;

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
        validate_invoice_reply_markup("sendInvoice", self.reply_markup.as_ref())?;
        validate_invoice_title("sendInvoice", &self.title)?;
        validate_invoice_description("sendInvoice", &self.description)?;
        validate_invoice_payload("sendInvoice", &self.payload)?;
        validate_invoice_currency("sendInvoice", &self.currency)?;
        validate_invoice_prices("sendInvoice", &self.currency, &self.prices)?;
        validate_invoice_tip_configuration(
            "sendInvoice",
            &self.currency,
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

    pub fn reply_markup(mut self, reply_markup: impl Into<InlineKeyboardMarkup>) -> Self {
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

        validate_invoice_title("createInvoiceLink", &title)?;
        validate_invoice_description("createInvoiceLink", &description)?;
        validate_invoice_payload("createInvoiceLink", &payload)?;
        validate_invoice_currency("createInvoiceLink", &currency)?;
        validate_invoice_prices("createInvoiceLink", &currency, &prices)?;

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
        validate_invoice_business_connection_id(
            "createInvoiceLink",
            &self.currency,
            self.business_connection_id.as_deref(),
        )?;
        validate_invoice_title("createInvoiceLink", &self.title)?;
        validate_invoice_description("createInvoiceLink", &self.description)?;
        validate_invoice_payload("createInvoiceLink", &self.payload)?;
        validate_invoice_currency("createInvoiceLink", &self.currency)?;
        validate_invoice_prices("createInvoiceLink", &self.currency, &self.prices)?;
        validate_invoice_tip_configuration(
            "createInvoiceLink",
            &self.currency,
            self.max_tip_amount,
            self.suggested_tip_amounts.as_deref(),
        )?;
        validate_invoice_subscription_period(
            "createInvoiceLink",
            &self.currency,
            self.subscription_period.map(i64::from),
            &self.prices,
        )?;
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

    fn valid_suggested_post_send_date() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(600, |duration| duration.as_secs() as i64 + 600)
    }

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
        invoice.reply_markup = Some(crate::types::telegram::InlineKeyboardMarkup::new(Vec::new()));
        assert!(matches!(
            invoice.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        invoice.reply_markup = Some(crate::types::telegram::InlineKeyboardMarkup::single_row(
            vec![crate::types::telegram::InlineKeyboardButton::callback(
                "Open", "invoice",
            )?],
        ));
        assert!(matches!(
            invoice.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        invoice.reply_markup = Some(crate::types::telegram::InlineKeyboardMarkup::single_row(
            vec![crate::types::telegram::InlineKeyboardButton::pay("Pay")],
        ));
        invoice.validate()?;
        invoice.reply_markup = None;
        invoice.message_effect_id = Some("bad\nid".to_owned());
        assert!(matches!(
            invoice.validate(),
            Err(Error::InvalidRequest { .. })
        ));
        invoice.message_effect_id = None;
        invoice.suggested_post_parameters =
            Some(SuggestedPostParameters::new(serde_json::json!({
                "send_date": valid_suggested_post_send_date()
            }))?);
        invoice.validate()?;
        let serialized =
            serde_json::to_value(&invoice).map_err(|source| Error::SerializeRequest { source })?;
        assert!(serialized.get("suggested_post_parameters").is_some());

        let stars_invoice = SendInvoiceRequest::new(
            1_i64,
            "title",
            "description",
            "payload",
            "XTR",
            vec![
                LabeledPrice::new("item", 100),
                LabeledPrice::new("shipping", 10),
            ],
        );
        assert!(matches!(stars_invoice, Err(Error::InvalidRequest { .. })));

        let mut stars_invoice = SendInvoiceRequest::new(
            1_i64,
            "title",
            "description",
            "payload",
            "XTR",
            vec![LabeledPrice::new("item", 100)],
        )?;
        stars_invoice.max_tip_amount = Some(10);
        assert!(matches!(
            stars_invoice.validate(),
            Err(Error::InvalidRequest { .. })
        ));

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
        invoice_link.business_connection_id = Some("business-1".to_owned());
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
        assert!(matches!(
            invoice_link.validate(),
            Err(Error::InvalidRequest { .. })
        ));

        let mut invoice_link = CreateInvoiceLinkRequest::new(
            "title",
            "description",
            "payload",
            "XTR",
            vec![LabeledPrice::new("item", 100)],
        )?;
        invoice_link.business_connection_id = Some("business-1".to_owned());
        invoice_link.subscription_period = Some(TELEGRAM_STARS_SUBSCRIPTION_PERIOD_SECS);
        invoice_link.validate()?;

        invoice_link.prices = vec![LabeledPrice::new("item", 10_001)];
        assert!(matches!(
            invoice_link.validate(),
            Err(Error::InvalidRequest { .. })
        ));

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
