// Integration tests with mock vsocket
// These tests simulate the full request/response flow without requiring Linux

use async_trait::async_trait;
use orbs_tee_nitro::{AppError, EnclaveApp, Response};
use orbs_tee_protocol::{TeeRequest, TeeResponse};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

/// Mock application for testing
struct MockApp {
    request_count: Arc<Mutex<usize>>,
}

impl MockApp {
    fn new() -> Self {
        Self {
            request_count: Arc::new(Mutex::new(0)),
        }
    }

    async fn get_request_count(&self) -> usize {
        *self.request_count.lock().await
    }
}

#[async_trait]
impl EnclaveApp for MockApp {
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    async fn handle_request(&self, method: &str, params: Value) -> Result<Response, AppError> {
        // Increment request counter
        *self.request_count.lock().await += 1;

        match method {
            "echo" => {
                // Echo back the params
                Ok(Response {
                    data: params,
                    sign: true,
                })
            }
            "get_price" => {
                // Simulate price oracle
                let symbol = params
                    .get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("BTCUSDT");

                Ok(Response {
                    data: json!({
                        "symbol": symbol,
                        "price": "42000.50",
                        "timestamp": 1234567890,
                    }),
                    sign: true,
                })
            }
            "error_test" => {
                // Intentionally return an error
                Err(AppError::InvalidRequest("Test error".to_string()))
            }
            "no_sign" => {
                // Response without signature
                Ok(Response {
                    data: json!({"message": "unsigned"}),
                    sign: false,
                })
            }
            _ => Err(AppError::InvalidRequest(format!(
                "Unknown method: {}",
                method
            ))),
        }
    }
}

/// Mock vsocket server using Unix domain sockets for testing
/// This simulates vsocket communication without needing Linux
struct MockVsockServer {
    socket_path: String,
}

impl MockVsockServer {
    fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    /// Start listening and handling requests (like real VsockServer)
    async fn listen<F, Fut>(&self, handler: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Vec<u8>> + Send,
    {
        // Remove socket file if it exists
        let _ = std::fs::remove_file(&self.socket_path);

        let listener = UnixListener::bind(&self.socket_path)?;

        let handler = Arc::new(handler);

        // Spawn server task
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let handler = handler.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, handler).await {
                                eprintln!("Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn handle_connection<F, Fut>(
        mut stream: UnixStream,
        handler: Arc<F>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(Vec<u8>) -> Fut,
        Fut: std::future::Future<Output = Vec<u8>>,
    {
        // Read request
        let request = Self::read_message(&mut stream).await?;

        // Process request
        let response = handler(request).await;

        // Write response
        Self::write_message(&mut stream, &response).await?;

        Ok(())
    }

    async fn read_message(stream: &mut UnixStream) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Read 4-byte length prefix
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        // Read message
        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer).await?;

        Ok(buffer)
    }

    async fn write_message(
        stream: &mut UnixStream,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Write length prefix
        let len = data.len() as u32;
        stream.write_all(&len.to_be_bytes()).await?;

        // Write message
        stream.write_all(data).await?;

        stream.flush().await?;

        Ok(())
    }
}

/// Mock client for sending requests to the mock server
struct MockVsockClient {
    socket_path: String,
}

impl MockVsockClient {
    fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    async fn send_request(&self, request: &TeeRequest) -> Result<TeeResponse, String> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| e.to_string())?;

        // Serialize and send request
        let request_bytes = serde_json::to_vec(request).map_err(|e| e.to_string())?;
        MockVsockServer::write_message(&mut stream, &request_bytes)
            .await
            .map_err(|e| e.to_string())?;

        // Read response
        let response_bytes = MockVsockServer::read_message(&mut stream)
            .await
            .map_err(|e| e.to_string())?;
        let response: TeeResponse =
            serde_json::from_slice(&response_bytes).map_err(|e| e.to_string())?;

        Ok(response)
    }
}

// Helper to create a simple request handler that works like EnclaveRuntime
fn create_mock_handler(
    app: Arc<Mutex<MockApp>>,
) -> impl Fn(Vec<u8>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<u8>> + Send>>
       + Send
       + Sync
       + Clone {
    move |request_bytes: Vec<u8>| {
        let app = app.clone();
        Box::pin(async move {
            // Deserialize request
            let request: TeeRequest = match serde_json::from_slice(&request_bytes) {
                Ok(req) => req,
                Err(e) => {
                    let error_response = TeeResponse {
                        id: "unknown".to_string(),
                        success: false,
                        data: None,
                        signature: None,
                        error: Some(format!("Failed to parse request: {}", e)),
                    };
                    return serde_json::to_vec(&error_response).unwrap_or_default();
                }
            };

            // Handle request
            let app_response = {
                let app = app.lock().await;
                app.handle_request(&request.method, request.params.clone())
                    .await
            };

            // Build response
            let response = match app_response {
                Ok(app_resp) => TeeResponse {
                    id: request.id.clone(),
                    success: true,
                    data: Some(app_resp.data),
                    signature: if app_resp.sign {
                        Some("mock_signature_here".to_string())
                    } else {
                        None
                    },
                    error: None,
                },
                Err(e) => TeeResponse {
                    id: request.id.clone(),
                    success: false,
                    data: None,
                    signature: None,
                    error: Some(e.to_string()),
                },
            };

            serde_json::to_vec(&response).unwrap_or_default()
        })
    }
}

// Tests

#[tokio::test]
async fn test_basic_request_response_flow() {
    let socket_path = "/tmp/orbs-tee-test-basic.sock".to_string();

    // Create mock app and handler
    let app = Arc::new(Mutex::new(MockApp::new()));
    let handler = create_mock_handler(app.clone());

    // Start mock server
    let server = MockVsockServer::new(socket_path.clone());
    server.listen(handler).await.unwrap();

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create client and send request
    let client = MockVsockClient::new(socket_path.clone());
    let request = TeeRequest {
        id: "test-1".to_string(),
        method: "echo".to_string(),
        params: json!({"message": "hello"}),
        timestamp: 1234567890,
    };

    let response = client.send_request(&request).await.unwrap();

    // Verify response
    assert_eq!(response.id, "test-1");
    assert!(response.success);
    assert_eq!(response.data, Some(json!({"message": "hello"})));
    assert!(response.signature.is_some()); // Should be signed
    assert!(response.error.is_none());

    // Verify request was counted
    let count = app.lock().await.get_request_count().await;
    assert_eq!(count, 1);

    // Cleanup
    let _ = std::fs::remove_file(&socket_path);
}

#[tokio::test]
async fn test_get_price_method() {
    let socket_path = "/tmp/orbs-tee-test-price.sock".to_string();

    let app = Arc::new(Mutex::new(MockApp::new()));
    let handler = create_mock_handler(app.clone());

    let server = MockVsockServer::new(socket_path.clone());
    server.listen(handler).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = MockVsockClient::new(socket_path.clone());
    let request = TeeRequest {
        id: "price-1".to_string(),
        method: "get_price".to_string(),
        params: json!({"symbol": "ETHUSDT"}),
        timestamp: 1234567890,
    };

    let response = client.send_request(&request).await.unwrap();

    assert!(response.success);
    assert!(response.signature.is_some());

    let data = response.data.unwrap();
    assert_eq!(data["symbol"], "ETHUSDT");
    assert_eq!(data["price"], "42000.50");

    let _ = std::fs::remove_file(&socket_path);
}

#[tokio::test]
async fn test_error_handling() {
    let socket_path = "/tmp/orbs-tee-test-error.sock".to_string();

    let app = Arc::new(Mutex::new(MockApp::new()));
    let handler = create_mock_handler(app);

    let server = MockVsockServer::new(socket_path.clone());
    server.listen(handler).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = MockVsockClient::new(socket_path.clone());
    let request = TeeRequest {
        id: "error-1".to_string(),
        method: "error_test".to_string(),
        params: json!({}),
        timestamp: 1234567890,
    };

    let response = client.send_request(&request).await.unwrap();

    // Should be error response
    assert!(!response.success);
    assert!(response.data.is_none());
    assert!(response.signature.is_none()); // No signature on errors
    assert!(response.error.is_some());
    assert!(response.error.unwrap().contains("Test error"));

    let _ = std::fs::remove_file(&socket_path);
}

#[tokio::test]
async fn test_unsigned_response() {
    let socket_path = "/tmp/orbs-tee-test-unsigned.sock".to_string();

    let app = Arc::new(Mutex::new(MockApp::new()));
    let handler = create_mock_handler(app);

    let server = MockVsockServer::new(socket_path.clone());
    server.listen(handler).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = MockVsockClient::new(socket_path.clone());
    let request = TeeRequest {
        id: "nosign-1".to_string(),
        method: "no_sign".to_string(),
        params: json!({}),
        timestamp: 1234567890,
    };

    let response = client.send_request(&request).await.unwrap();

    assert!(response.success);
    assert!(response.signature.is_none()); // Should not be signed
    assert_eq!(response.data, Some(json!({"message": "unsigned"})));

    let _ = std::fs::remove_file(&socket_path);
}

#[tokio::test]
async fn test_concurrent_requests() {
    let socket_path = "/tmp/orbs-tee-test-concurrent.sock".to_string();

    let app = Arc::new(Mutex::new(MockApp::new()));
    let handler = create_mock_handler(app.clone());

    let server = MockVsockServer::new(socket_path.clone());
    server.listen(handler).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Send 10 concurrent requests
    let mut handles = vec![];
    for i in 0..10 {
        let socket_path = socket_path.clone();
        let handle = tokio::spawn(async move {
            let client = MockVsockClient::new(socket_path);
            let request = TeeRequest {
                id: format!("concurrent-{}", i),
                method: "echo".to_string(),
                params: json!({"request_num": i}),
                timestamp: 1234567890,
            };
            client.send_request(&request).await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(response)) = handle.await {
            assert!(response.success);
            success_count += 1;
        }
    }

    // All requests should succeed
    assert_eq!(success_count, 10);

    // Verify all requests were counted
    let count = app.lock().await.get_request_count().await;
    assert_eq!(count, 10);

    let _ = std::fs::remove_file(&socket_path);
}

#[tokio::test]
async fn test_unknown_method() {
    let socket_path = "/tmp/orbs-tee-test-unknown.sock".to_string();

    let app = Arc::new(Mutex::new(MockApp::new()));
    let handler = create_mock_handler(app);

    let server = MockVsockServer::new(socket_path.clone());
    server.listen(handler).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = MockVsockClient::new(socket_path.clone());
    let request = TeeRequest {
        id: "unknown-1".to_string(),
        method: "nonexistent_method".to_string(),
        params: json!({}),
        timestamp: 1234567890,
    };

    let response = client.send_request(&request).await.unwrap();

    assert!(!response.success);
    assert!(response.error.is_some());
    assert!(response
        .error
        .unwrap()
        .contains("Unknown method: nonexistent_method"));

    let _ = std::fs::remove_file(&socket_path);
}

#[tokio::test]
async fn test_malformed_request() {
    let socket_path = "/tmp/orbs-tee-test-malformed.sock".to_string();

    let app = Arc::new(Mutex::new(MockApp::new()));
    let handler = create_mock_handler(app);

    let server = MockVsockServer::new(socket_path.clone());
    server.listen(handler).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Send invalid JSON
    let mut stream = UnixStream::connect(&socket_path).await.unwrap();
    let invalid_json = b"this is not json";
    MockVsockServer::write_message(&mut stream, invalid_json)
        .await
        .unwrap();

    // Read response
    let response_bytes = MockVsockServer::read_message(&mut stream).await.unwrap();
    let response: TeeResponse = serde_json::from_slice(&response_bytes).unwrap();

    // Should get error response
    assert!(!response.success);
    assert!(response.error.is_some());
    assert!(response.error.unwrap().contains("Failed to parse request"));

    let _ = std::fs::remove_file(&socket_path);
}
