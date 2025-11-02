// Integration tests for crypto module

use orbs_tee_nitro::crypto::KeyManager;
use secp256k1::{Message, Secp256k1};
use serde_json::json;
use sha2::{Digest, Sha256};

#[test]
fn test_key_generation() {
    // Should successfully generate a new key pair
    let key_manager = KeyManager::new();
    assert!(key_manager.is_ok(), "Key generation should succeed");
}

#[test]
fn test_public_key_bytes_format() {
    let key_manager = KeyManager::new().unwrap();
    let public_key_bytes = key_manager.public_key_bytes();

    // Compressed public key should be exactly 33 bytes
    assert_eq!(
        public_key_bytes.len(),
        33,
        "Compressed public key should be 33 bytes"
    );

    // First byte should be 0x02 or 0x03 (compressed format prefix)
    assert!(
        public_key_bytes[0] == 0x02 || public_key_bytes[0] == 0x03,
        "First byte should be 0x02 or 0x03 for compressed format"
    );
}

#[test]
fn test_public_key_hex_format() {
    let key_manager = KeyManager::new().unwrap();
    let public_key_hex = key_manager.public_key_hex();

    // Should start with "0x"
    assert!(
        public_key_hex.starts_with("0x"),
        "Public key hex should start with 0x"
    );

    // Should be 68 characters total (0x + 66 hex chars for 33 bytes)
    assert_eq!(
        public_key_hex.len(),
        68,
        "Public key hex should be 68 characters"
    );

    // Should be valid hex after the 0x prefix
    let hex_part = &public_key_hex[2..];
    assert!(
        hex_part.chars().all(|c| c.is_ascii_hexdigit()),
        "Should be valid hex"
    );
}

#[test]
fn test_sign_data() {
    let key_manager = KeyManager::new().unwrap();
    let data = b"Hello, World!";

    let signature = key_manager.sign(data);
    assert!(signature.is_ok(), "Signing should succeed");

    let sig = signature.unwrap();
    // ECDSA compact signature should be exactly 64 bytes (32 bytes r + 32 bytes s)
    assert_eq!(sig.len(), 64, "Signature should be 64 bytes");
}

#[test]
fn test_sign_json() {
    let key_manager = KeyManager::new().unwrap();
    let json_data = json!({
        "symbol": "BTCUSDT",
        "price": "50000.00",
        "timestamp": 1234567890
    });

    let signature = key_manager.sign_json(&json_data);
    assert!(signature.is_ok(), "JSON signing should succeed");

    let sig = signature.unwrap();
    assert_eq!(sig.len(), 64, "Signature should be 64 bytes");
}

#[test]
fn test_signature_verification() {
    let key_manager = KeyManager::new().unwrap();
    let data = b"Test message for verification";

    // Sign the data
    let signature = key_manager.sign(data).unwrap();

    // Verify the signature using secp256k1
    let secp = Secp256k1::new();

    // Hash the data (same as in sign method)
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    let message = Message::from_digest_slice(&hash).unwrap();

    // Parse the signature
    let sig = secp256k1::ecdsa::Signature::from_compact(&signature).unwrap();

    // Get the public key
    let public_key_bytes = key_manager.public_key_bytes();
    let public_key = secp256k1::PublicKey::from_slice(&public_key_bytes).unwrap();

    // Verify signature with public key
    let verification = secp.verify_ecdsa(&message, &sig, &public_key);
    assert!(verification.is_ok(), "Signature should be valid");
}

#[test]
fn test_deterministic_signing() {
    let key_manager = KeyManager::new().unwrap();
    let data = b"Deterministic test";

    // Sign the same data twice
    let sig1 = key_manager.sign(data).unwrap();
    let sig2 = key_manager.sign(data).unwrap();

    // ECDSA signatures should be deterministic with the same key and data
    assert_eq!(sig1, sig2, "Signatures should be deterministic");
}

#[test]
fn test_different_data_different_signatures() {
    let key_manager = KeyManager::new().unwrap();
    let data1 = b"Message 1";
    let data2 = b"Message 2";

    let sig1 = key_manager.sign(data1).unwrap();
    let sig2 = key_manager.sign(data2).unwrap();

    // Different data should produce different signatures
    assert_ne!(
        sig1, sig2,
        "Different data should have different signatures"
    );
}

#[test]
fn test_clone() {
    let key_manager = KeyManager::new().unwrap();
    let cloned = key_manager.clone();

    // Cloned key manager should have the same public key
    assert_eq!(
        key_manager.public_key_bytes(),
        cloned.public_key_bytes(),
        "Cloned key manager should have same public key"
    );

    // Should be able to sign with both
    let data = b"Clone test";
    let sig1 = key_manager.sign(data).unwrap();
    let sig2 = cloned.sign(data).unwrap();

    // Both should produce the same signature
    assert_eq!(
        sig1, sig2,
        "Cloned key manager should produce same signatures"
    );
}

#[test]
fn test_unique_keys_per_instance() {
    let key_manager1 = KeyManager::new().unwrap();
    let key_manager2 = KeyManager::new().unwrap();

    // Different instances should have different keys
    assert_ne!(
        key_manager1.public_key_bytes(),
        key_manager2.public_key_bytes(),
        "Each instance should have unique keys"
    );
}

#[test]
fn test_json_canonicalization() {
    use serde_json::json;

    let key_manager = KeyManager::new().unwrap();

    // Create JSON with different key orderings - should produce identical signatures
    let json1 = json!({
        "z_last": "value3",
        "a_first": "value1",
        "m_middle": "value2"
    });

    let json2 = json!({
        "a_first": "value1",
        "m_middle": "value2",
        "z_last": "value3"
    });

    let json3 = json!({
        "m_middle": "value2",
        "z_last": "value3",
        "a_first": "value1"
    });

    // All three should produce the same signature despite different key ordering
    let sig1 = key_manager.sign_json(&json1).unwrap();
    let sig2 = key_manager.sign_json(&json2).unwrap();
    let sig3 = key_manager.sign_json(&json3).unwrap();

    assert_eq!(
        sig1, sig2,
        "Signatures should be identical regardless of key order"
    );
    assert_eq!(
        sig2, sig3,
        "Signatures should be identical regardless of key order"
    );
}

#[test]
fn test_nested_json_canonicalization() {
    use serde_json::json;

    let key_manager = KeyManager::new().unwrap();

    // Test nested objects with different key orderings
    let json1 = json!({
        "outer_z": {
            "inner_z": "value2",
            "inner_a": "value1"
        },
        "outer_a": "value3"
    });

    let json2 = json!({
        "outer_a": "value3",
        "outer_z": {
            "inner_a": "value1",
            "inner_z": "value2"
        }
    });

    let sig1 = key_manager.sign_json(&json1).unwrap();
    let sig2 = key_manager.sign_json(&json2).unwrap();

    assert_eq!(
        sig1, sig2,
        "Nested JSON signatures should be identical regardless of key order"
    );
}
