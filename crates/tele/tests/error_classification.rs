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
fn classifies_local_auth_errors() {
    for error in [Error::InvalidBotToken, Error::MissingBotToken] {
        assert_eq!(error.classification(), ErrorClass::Authentication);
        assert!(error.is_auth_error());
        assert!(!error.is_retryable());
    }
}

#[test]
fn classifies_auth_errors_from_api_status_without_error_code() {
    let error = Error::Api {
        method: "getMe".to_owned(),
        status: Some(403),
        request_id: None,
        error_code: None,
        description: "forbidden".into(),
        parameters: None,
        body_snippet: None,
    };

    assert_eq!(error.classification(), ErrorClass::Authentication);
    assert!(error.is_auth_error());
    assert!(!error.is_retryable());
}

#[test]
fn auth_transport_statuses_are_not_retryable() {
    let error = Error::Transport {
        method: "getMe".to_owned(),
        status: Some(401),
        request_id: None,
        retry_after: None,
        request_path: None,
        message: "unauthorized".into(),
    };

    assert_eq!(error.classification(), ErrorClass::Authentication);
    assert!(error.is_auth_error());
    assert!(!error.is_retryable());
}

#[test]
fn non_retryable_transport_status_is_not_retryable() {
    let error = Error::Transport {
        method: "sendMessage".to_owned(),
        status: Some(400),
        request_id: None,
        retry_after: None,
        request_path: None,
        message: "bad request".into(),
    };

    assert_eq!(error.classification(), ErrorClass::Transport);
    assert!(!error.is_retryable());
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
fn classifies_rate_limit_from_api_status_without_error_code() {
    let error = Error::Api {
        method: "sendMessage".to_owned(),
        status: Some(429),
        request_id: None,
        error_code: None,
        description: "too many requests".into(),
        parameters: None,
        body_snippet: None,
    };

    assert_eq!(error.classification(), ErrorClass::RateLimited);
    assert!(error.is_rate_limited());
    assert!(error.is_retryable());
}

#[test]
fn configuration_errors_are_not_retryable() {
    let error = Error::Configuration {
        reason: "invalid proxy config".to_owned(),
    };

    assert_eq!(error.classification(), ErrorClass::Configuration);
    assert!(!error.is_retryable());
}

#[test]
fn storage_errors_preserve_retry_policy() {
    let retryable = Error::Storage {
        operation: "redis GET".into(),
        message: "connection reset".into(),
        retryable: true,
    };
    assert_eq!(retryable.classification(), ErrorClass::Storage);
    assert!(retryable.is_retryable());

    let permanent = Error::Storage {
        operation: "postgres decode".into(),
        message: "invalid json payload".into(),
        retryable: false,
    };
    assert_eq!(permanent.classification(), ErrorClass::Storage);
    assert!(!permanent.is_retryable());
}
