// This file handles AWS Nitro Secure Module (NSM) attestation
// This is Nitro-specific implementation

use aws_nitro_enclaves_nsm_api::{
    api::{Request, Response},
    driver::{nsm_init, nsm_process_request},
};
use serde_bytes::ByteBuf;

/// Errors that can occur during attestation
#[derive(Debug, thiserror::Error)]
pub enum NitroError {
    #[error("Failed to initialize NSM device")]
    InitFailed,
    
    #[error("Failed to generate attestation: {0}")]
    GenerationFailed(String),
    
    #[error("Unexpected response from NSM")]
    UnexpectedResponse,
}

/// Handles communication with the Nitro Secure Module (NSM)
/// NSM is a hardware device inside the Nitro Enclave that provides attestation
pub struct NitroAttestation {
    /// File descriptor for /dev/nsm device
    /// This is how we talk to the NSM hardware
    nsm_fd: i32,
}

impl NitroAttestation {
    /// Initialize connection to NSM device
    /// This opens /dev/nsm and returns a file descriptor
    pub fn new() -> Result<Self, NitroError> {
        // nsm_init() opens /dev/nsm device and returns file descriptor
        // Returns -1 on error (C convention)
        let nsm_fd = unsafe { nsm_init() };
        
        if nsm_fd < 0 {
            return Err(NitroError::InitFailed);
        }
        
        Ok(Self { nsm_fd })
    }
    
    /// Generate an attestation document
    /// 
    /// Parameters:
    /// - public_key: Enclave's public key to embed in attestation
    /// - user_data: Optional custom data (e.g., nonce, app-specific info)
    /// - nonce: Optional nonce for freshness
    /// 
    /// Returns: Raw attestation document (CBOR encoded)
    /// 
    /// The attestation document contains:
    /// - PCRs (Platform Configuration Registers) - hashes of enclave code
    /// - Public key embedded in the document
    /// - User data
    /// - Certificate chain signed by AWS
    /// - Timestamp
    pub fn generate_attestation(
        &self,
        public_key: Vec<u8>,
        user_data: Option<Vec<u8>>,
        nonce: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, NitroError> {
        // Create attestation request for NSM
        // Note: NSM API expects ByteBuf (from serde_bytes) instead of Vec<u8>
        let request = Request::Attestation {
            // Public key to include in attestation (proves this key belongs to this enclave)
            public_key: Some(ByteBuf::from(public_key)),

            // Optional user data (custom application data)
            user_data: user_data.map(ByteBuf::from),

            // Optional nonce (prevents replay attacks)
            nonce: nonce.map(ByteBuf::from),
        };
        
        // Send request to NSM device and get response
        // This is a synchronous call to the NSM hardware
        let response = unsafe {
            nsm_process_request(self.nsm_fd, request)
        };
        
        // Parse the response
        match response {
            // Success - return the attestation document
            Response::Attestation { document } => Ok(document),
            
            // Error from NSM
            Response::Error(err) => {
                Err(NitroError::GenerationFailed(format!("{:?}", err)))
            }
            
            // Unexpected response type
            _ => Err(NitroError::UnexpectedResponse),
        }
    }
}

// Clean up NSM file descriptor when dropped
impl Drop for NitroAttestation {
    fn drop(&mut self) {
        // Close the NSM device file descriptor
        // This is important to avoid resource leaks
        unsafe {
            libc::close(self.nsm_fd);
        }
    }
}
