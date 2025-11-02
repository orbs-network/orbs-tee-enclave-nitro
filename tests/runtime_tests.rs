// Integration tests for EnclaveRuntime

use async_trait::async_trait;
use orbs_tee_nitro::{AppError, EnclaveApp, Response};
use serde_json::json;

// Test application implementation
struct TestApp {
    initialized: bool,
}

impl TestApp {
    fn new() -> Self {
        Self { initialized: false }
    }
}

#[async_trait]
impl EnclaveApp for TestApp {
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.initialized = true;
        Ok(())
    }

    async fn handle_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Response, AppError> {
        match method {
            "echo" => Ok(Response {
                data: params,
                sign: false,
            }),
            "sign_echo" => Ok(Response {
                data: params,
                sign: true,
            }),
            "error" => Err(AppError::InternalError("Test error".to_string())),
            _ => Err(AppError::InvalidRequest(format!(
                "Unknown method: {}",
                method
            ))),
        }
    }
}

#[tokio::test]
async fn test_app_initialization() {
    let mut app = TestApp::new();
    assert!(!app.initialized, "App should not be initialized yet");

    app.init().await.unwrap();
    assert!(app.initialized, "App should be initialized after init()");
}

#[tokio::test]
async fn test_echo_request() {
    let app = TestApp::new();
    let response = app
        .handle_request("echo", json!({"message": "hello"}))
        .await;

    assert!(response.is_ok());
    let resp = response.unwrap();
    assert_eq!(resp.data["message"], "hello");
    assert!(!resp.sign, "Echo should not be signed");
}

#[tokio::test]
async fn test_signed_echo_request() {
    let app = TestApp::new();
    let response = app
        .handle_request("sign_echo", json!({"data": "sign me"}))
        .await;

    assert!(response.is_ok());
    let resp = response.unwrap();
    assert_eq!(resp.data["data"], "sign me");
    assert!(resp.sign, "Sign echo should be signed");
}

#[tokio::test]
async fn test_error_request() {
    let app = TestApp::new();
    let response = app.handle_request("error", json!({})).await;

    assert!(response.is_err());
    let err = response.unwrap_err();
    match err {
        AppError::InternalError(msg) => {
            assert_eq!(msg, "Test error");
        }
        _ => panic!("Expected InternalError"),
    }
}

#[tokio::test]
async fn test_invalid_method() {
    let app = TestApp::new();
    let response = app.handle_request("unknown_method", json!({})).await;

    assert!(response.is_err());
    let err = response.unwrap_err();
    match err {
        AppError::InvalidRequest(msg) => {
            assert!(msg.contains("Unknown method"));
        }
        _ => panic!("Expected InvalidRequest"),
    }
}

#[tokio::test]
async fn test_app_error_display() {
    let errors = vec![
        AppError::InvalidRequest("test".to_string()),
        AppError::InternalError("test".to_string()),
        AppError::NetworkError("test".to_string()),
    ];

    for error in errors {
        let error_string = error.to_string();
        assert!(
            error_string.contains("test"),
            "Error should display message"
        );
    }
}
