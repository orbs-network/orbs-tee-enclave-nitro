// This is the main runtime that orchestrates everything
// It brings together: key management, Nitro attestation, vsocket server, and user's app

use crate::{
    EnclaveApp, Response,
    crypto::KeyManager,
    TeeRequest, TeeResponse,
};

#[cfg(feature = "nitro")]
use crate::AppError;

#[cfg(feature = "nitro")]
use crate::nitro::NitroAttestation;

use std::sync::Arc;
use tokio::sync::Mutex;

/// The main runtime that runs the enclave application
/// This is what brings everything together:
/// - Creates keys
/// - Sets up Nitro attestation
/// - Starts vsocket server
/// - Routes requests to your app
///
/// Generic parameter T: Your app that implements EnclaveApp trait
pub struct EnclaveRuntime<T: EnclaveApp> {
    /// Your application (implements EnclaveApp trait)
    #[allow(dead_code)]
    app: Arc<Mutex<T>>,

    /// Key manager for signing
    key_manager: KeyManager,

    /// Nitro attestation manager
    #[cfg(feature = "nitro")]
    nitro: NitroAttestation,
}

impl<T: EnclaveApp + 'static> EnclaveRuntime<T> {
    /// Create a new runtime with your app
    ///
    /// This:
    /// 1. Generates a new key pair
    /// 2. Initializes NSM for attestation (if nitro feature enabled)
    /// 3. Calls your app's init() method
    #[cfg(feature = "nitro")]
    pub async fn new(mut app: T) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🔐 Generating enclave key pair...");
        let key_manager = KeyManager::new()?;

        println!("📝 Public key: {}", key_manager.public_key_hex());

        println!("🔒 Initializing Nitro Secure Module...");
        let nitro = NitroAttestation::new()?;

        println!("🚀 Initializing application...");
        app.init().await?;

        Ok(Self {
            app: Arc::new(Mutex::new(app)),
            key_manager,
            nitro,
        })
    }

    /// Create a new runtime without Nitro features (for testing)
    #[cfg(not(feature = "nitro"))]
    pub async fn new(mut app: T) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🔐 Generating enclave key pair...");
        let key_manager = KeyManager::new()?;

        println!("📝 Public key: {}", key_manager.public_key_hex());

        println!("🚀 Initializing application...");
        app.init().await?;

        Ok(Self {
            app: Arc::new(Mutex::new(app)),
            key_manager,
        })
    }

    /// Start the vsocket server and handle requests
    /// This runs forever, processing requests from the host
    #[cfg(feature = "nitro")]
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::vsock::VsockServer;

        println!("👂 Starting vsocket server on port 5000...");
        let server = VsockServer::new(5000);

        let runtime = Arc::new(self);

        server.listen(move |request_bytes| {
            let runtime = runtime.clone();
            async move {
                runtime.handle_request(request_bytes).await
            }
        }).await?;

        Ok(())
    }

    /// Handle a single request from the host
    async fn handle_request(&self, request_bytes: Vec<u8>) -> Vec<u8> {
        // Deserialize the request
        let request: TeeRequest = match serde_json::from_slice(&request_bytes) {
            Ok(req) => req,
            Err(e) => {
                let error_response = TeeResponse::error(
                    "unknown".to_string(),
                    format!("Failed to parse request: {}", e),
                );
                return serde_json::to_vec(&error_response).unwrap_or_default();
            }
        };

        println!("📨 Received request: {} - {}", request.id, request.method);

        // Route to application handler
        let app_response = {
            let app = self.app.lock().await;
            app.handle_request(&request.method, request.params.clone()).await
        };

        // Build response
        let mut response = match &app_response {
            Ok(app_resp) => TeeResponse::success(request.id.clone(), app_resp.data.clone()),
            Err(e) => TeeResponse::error(request.id.clone(), e.to_string()),
        };

        // Sign response if requested
        if let Ok(Response { sign: true, .. }) = app_response {
            if let Some(ref data) = response.data {
                match self.key_manager.sign_json(data) {
                    Ok(sig) => {
                        response.signature = Some(hex::encode(sig));
                        println!("✍️  Signed response for request {}", request.id);
                    }
                    Err(e) => {
                        println!("⚠️  Failed to sign response: {}", e);
                    }
                }
            }
        }

        println!("📤 Sending response for request {}", request.id);

        // Serialize and return
        // If serialization fails, return a minimal error response instead of empty bytes
        match serde_json::to_vec(&response) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("❌ CRITICAL: Failed to serialize response: {}", e);
                // Return a minimal error response that should always serialize
                let error = TeeResponse::error(
                    request.id.clone(),
                    format!("Internal serialization error: {}", e),
                );
                serde_json::to_vec(&error).unwrap_or_default()
            }
        }
    }

    /// Get the enclave's public key
    pub fn public_key(&self) -> String {
        self.key_manager.public_key_hex()
    }

    /// Generate an attestation document with the enclave's public key
    #[cfg(feature = "nitro")]
    pub fn get_attestation(
        &self,
        user_data: Option<Vec<u8>>,
        nonce: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let public_key = self.key_manager.public_key_bytes();
        Ok(self.nitro.generate_attestation(public_key, user_data, nonce)?)
    }
}