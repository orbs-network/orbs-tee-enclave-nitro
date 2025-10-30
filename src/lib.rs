// This file defines the public API that DApp developers will use
// This is the Nitro-specific SDK

// Re-export everything from submodules
pub mod app;
pub mod nitro;      // Nitro-specific attestation
pub mod crypto;
pub mod vsock;
pub mod protocol;

// Import the main types to re-export them
use app::EnclaveRuntime;
use protocol::{TeeRequest, TeeResponse};

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
    /// 
    /// Example:
    ///
