use std::time::Duration;

use tele::types::ResponseParameters;
use tele::{Error, ErrorClass};

#[test]
fn classifies_rate_limited_transport_errors() {
    let error = Error::Transport {
        method: "sendMessage".to_owned(),
        status: Some(429),
        request_id: None,
        retry_after: Some(Duration::from_secs(3)),
        request_path: None,
        message: "too many requests".into(),
    };

    assert_eq!(error.classification(), ErrorClass::RateLimited);
    assert!(error.is_rate_limited());
    assert!(error.is_retryable());
}

#[test]
fn classifies_auth_errors_from_api_code() {
    let error = Error::Api {
        method: "getMe".to_owned(),
        status: Some(401),
        request_id: None,
        error_code: Some(401),
        description: "unauthorized".into(),
        parameters: None,
        body_snippet: None,
    };

    assert_eq!(error.classification(), ErrorClass::Authentication);
    assert!(error.is_auth_error());
}

#[test]
fn classifies_protocol_and_decode_errors() {
    let missing_result = Error::MissingResult {
        method: "getMe".to_owned(),
        status: Some(200),
        request_id: None,
        body_snippet: None,
    };
    assert_eq!(missing_result.classification(), ErrorClass::Protocol);

    let mut parameters = ResponseParameters::default();
    parameters.retry_after = Some(1);

    let api_with_retry = Error::Api {
        method: "sendMessage".to_owned(),
        status: Some(200),
        request_id: None,
        error_code: Some(400),
        description: "retry later".into(),
        parameters: Some(Box::new(parameters)),
        body_snippet: None,
    };
    assert_eq!(api_with_retry.classification(), ErrorClass::RateLimited);
    assert!(api_with_retry.is_rate_limited());
}

#[test]
fn configuration_errors_are_not_retryable() {
    let error = Error::Configuration {
        reason: "invalid proxy config".to_owned(),
    };

    assert_eq!(error.classification(), ErrorClass::Configuration);
    assert!(!error.is_retryable());
}
