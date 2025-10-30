// This file defines the public API that DApp developers will use
// This is the Nitro-specific SDK

// Re-export everything from submodules
pub mod app;
#[cfg(feature = "nitro")]
pub mod nitro;      // Nitro-specific attestation
pub mod crypto;
#[cfg(feature = "nitro")]
pub mod vsock;

// Re-export protocol types from the shared protocol package
pub use orbs_tee_protocol::{TeeRequest, TeeResponse};

// Import the main types to re-export them
#[cfg(feature = "nitro")]
use app::EnclaveRuntime;

// Error types that can be returned
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
}

// The main trait that DApp developers implement
// "trait" in Rust is like an interface in other languages
// #[async_trait] allows us to use async/await in trait methods
use async_trait::async_trait;

/// Main trait that TAPP developers implement
/// This defines the interface your DApp must provide
#[async_trait]
pub trait EnclaveApp: Send + Sync {
    /// Called once when the enclave starts up
    /// Use this to initialize any state your app needs
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Handle a request from the host
    /// method: The method name (e.g., "get_price", "hello")
    /// params: JSON parameters for the request
    /// Returns: Response with data and whether to sign it
    async fn handle_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Response, AppError>;
}

/// Response returned by EnclaveApp::handle_request
#[derive(Debug, Clone)]
pub struct Response {
    /// The response data (will be JSON-serialized)
    pub data: serde_json::Value,

    /// Whether to sign this response with the enclave's private key
    pub sign: bool,
}

/// Main entry point - run an enclave application
/// This creates the runtime and starts the vsocket server
#[cfg(feature = "nitro")]
pub async fn run_enclave_app<T: EnclaveApp + 'static>(
    app: T,
) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = EnclaveRuntime::new(app).await?;
    runtime.start().await
}
