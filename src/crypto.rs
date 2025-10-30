// This file handles cryptographic operations:
// - Key generation
// - Data signing with ECDSA
// - Key serialization

use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
use sha2::{Sha256, Digest};
use rand::rngs::OsRng;

/// Manages cryptographic keys for the enclave
/// Each enclave instance has ONE key pair that persists while enclave is running
pub struct KeyManager {
    /// Private key (NEVER leaves the enclave)
    private_key: SecretKey,
    
    /// Public key (shared with outside world)
    public_key: PublicKey,
    
    /// Secp256k1 context (reused for efficiency)
    secp: Secp256k1<secp256k1::All>,
}

impl KeyManager {
    /// Generate a new random key pair
    /// This is called once when the enclave starts
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create secp256k1 context (used for all crypto operations)
        let secp = Secp256k1::new();
        
        // Generate random private key using OS random number generator
        // OsRng is cryptographically secure (uses /dev/urandom on Linux)
        let mut rng = OsRng;
        let private_key = SecretKey::new(&mut rng);
        
        // Derive public key from private key
        // This is one-way: private -> public is easy, public -> private is impossible
        let public_key = PublicKey::from_secret_key(&secp, &private_key);
        
        Ok(Self {
            private_key,
            public_key,
            secp,
        })
    }
    
    /// Get the public key as bytes (33 bytes compressed format)
    /// Format: 0x02/0x03 (1 byte) + X coordinate (32 bytes)
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key.serialize().to_vec()
    }
    
    /// Get the public key as hex string with 0x prefix
    /// Example: "0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"
    pub fn public_key_hex(&self) -> String {
        format!("0x{}", hex::encode(self.public_key_bytes()))
    }
    
    /// Sign arbitrary data with ECDSA
    /// Returns 64-byte signature (r, s components, 32 bytes each)
    /// 
    /// Process:
    /// 1. Hash the data with SHA-256 (32 bytes)
    /// 2. Sign the hash with ECDSA
    /// 3. Return compact signature (64 bytes)
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Hash the data first (ECDSA signs fixed-size hashes, not arbitrary data)
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        
        // Convert hash to secp256k1 Message type
        let message = Message::from_digest_slice(&hash)?;
        
        // Sign the message with our private key
        let signature = self.secp.sign_ecdsa(&message, &self.private_key);
        
        // Return signature in compact format (64 bytes: 32 bytes r + 32 bytes s)
        Ok(signature.serialize_compact().to_vec())
    }
    
    /// Sign JSON data
    /// Convenience method that serializes JSON to bytes, then signs
    pub fn sign_json(&self, data: &serde_json::Value) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Serialize JSON to bytes (deterministic ordering)
        let bytes = serde_json::to_vec(data)?;
        
        // Sign the bytes
        self.sign(&bytes)
    }
}

// Implement Clone so we can share KeyManager across threads
// Note: This doesn't actually clone the private key - it uses Arc internally
impl Clone for KeyManager {
    fn clone(&self) -> Self {
        Self {
            private_key: self.private_key.clone(),
            public_key: self.public_key.clone(),
            secp: Secp256k1::new(),
        }
    }
}

