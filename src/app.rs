// This is the main runtime that orchestrates everything
// It brings together: key management, Nitro attestation, vsocket server, and user's app

use crate::{
    EnclaveApp, AppError, Response,
    crypto::KeyManager,
    nitro::NitroAttestation,
    TeeRequest, TeeResponse,
};
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
    app: Arc<Mutex<T>>,
    
    /// Key manager for signing
    key_manager: KeyManager,
    
    /// Nitro attestation manager
    nitro: NitroAttestation,
}

impl<T: EnclaveApp + 'static> EnclaveRuntime<T> {
    /// Create a new runtime with your app
    /// 
    /// This:
    /// 1. Generates a new key pair
    /// 2. Initializes NSM for attestation
    /// 3. Calls your app's init() method
    /// 
    /// Example:
    ///
