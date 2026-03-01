use serde::{Deserialize, Serialize};

use crate::Error;
use crate::types::common::ChatId;
use crate::types::telegram::{ReplyMarkup, ReplyParameters};

fn ensure_non_empty(method: &str, field: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(Error::InvalidRequest {
            reason: format!("{method} requires non-empty `{field}`"),
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
    pub business_connection_id: Option<String>,
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

        ensure_non_empty("sendInvoice", "title", &title)?;
        ensure_non_empty("sendInvoice", "description", &description)?;
        ensure_non_empty("sendInvoice", "payload", &payload)?;
        validate_currency("sendInvoice", &currency)?;
        validate_prices("sendInvoice", &prices)?;

        let request = Self {
            chat_id: chat_id.into(),
            title,
            description,
            payload,
            currency,
            prices,
            business_connection_id: None,
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
            reply_parameters: None,
            reply_markup: None,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> Result<(), Error> {
        ensure_non_empty("sendInvoice", "title", &self.title)?;
        ensure_non_empty("sendInvoice", "description", &self.description)?;
        ensure_non_empty("sendInvoice", "payload", &self.payload)?;
        validate_currency("sendInvoice", &self.currency)?;
        validate_prices("sendInvoice", &self.prices)?;
        validate_tip_configuration(
            "sendInvoice",
            self.max_tip_amount,
            self.suggested_tip_amounts.as_deref(),
        )?;
        Ok(())
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

        ensure_non_empty("createInvoiceLink", "title", &title)?;
        ensure_non_empty("createInvoiceLink", "description", &description)?;
        ensure_non_empty("createInvoiceLink", "payload", &payload)?;
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
        ensure_non_empty("createInvoiceLink", "title", &self.title)?;
        ensure_non_empty("createInvoiceLink", "description", &self.description)?;
        ensure_non_empty("createInvoiceLink", "payload", &self.payload)?;
        validate_currency("createInvoiceLink", &self.currency)?;
        validate_prices("createInvoiceLink", &self.prices)?;
        validate_tip_configuration(
            "createInvoiceLink",
            self.max_tip_amount,
            self.suggested_tip_amounts.as_deref(),
        )?;
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
        if self.ok {
            return Ok(());
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
        if self.ok {
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
