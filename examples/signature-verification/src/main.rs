// Signature Verification Example
//
// This example demonstrates how to verify signatures from ORBS TEE Nitro enclaves.
// The enclave signs responses with ECDSA (secp256k1 curve) + SHA-256.
//
// Use this code on the host side (outside the enclave) to verify that responses
// actually came from the enclave and haven't been tampered with.

use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Canonicalize JSON for deterministic serialization
///
/// IMPORTANT: This must match the canonicalization used by the enclave!
/// The enclave uses the same algorithm (sorted keys, compact format).
fn canonicalize_json(value: &Value) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    fn sort_value(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                // Convert to BTreeMap for sorted keys
                let sorted: BTreeMap<String, Value> = map
                    .iter()
                    .map(|(k, v)| (k.clone(), sort_value(v)))
                    .collect();
                serde_json::to_value(sorted).unwrap()
            }
            Value::Array(arr) => {
                // Recursively sort array elements
                Value::Array(arr.iter().map(sort_value).collect())
            }
            _ => value.clone(),
        }
    }

    // Sort the JSON structure
    let sorted = sort_value(value);

    // Serialize to compact JSON (no whitespace)
    Ok(serde_json::to_vec(&sorted)?)
}

/// Verify an ECDSA signature
///
/// Parameters:
/// - `data`: The original data that was signed (will be hashed with SHA-256)
/// - `signature_hex`: The signature as a hex string (without 0x prefix)
/// - `public_key_hex`: The public key as a hex string (with or without 0x prefix)
///
/// Returns: true if signature is valid, false otherwise
pub fn verify_signature(
    data: &[u8],
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Remove 0x prefix if present
    let signature_hex = signature_hex.strip_prefix("0x").unwrap_or(signature_hex);
    let public_key_hex = public_key_hex.strip_prefix("0x").unwrap_or(public_key_hex);

    // Decode hex strings
    let signature_bytes = hex::decode(signature_hex)?;
    let public_key_bytes = hex::decode(public_key_hex)?;

    // Parse signature (64 bytes: 32 bytes r + 32 bytes s)
    if signature_bytes.len() != 64 {
        return Err(format!(
            "Invalid signature length: expected 64 bytes, got {}",
            signature_bytes.len()
        )
        .into());
    }
    let signature = Signature::from_compact(&signature_bytes)?;

    // Parse public key (33 bytes compressed format)
    let public_key = PublicKey::from_slice(&public_key_bytes)?;

    // Hash the data with SHA-256
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    // Convert hash to secp256k1 Message
    let message = Message::from_digest_slice(&hash)?;

    // Verify the signature
    let secp = Secp256k1::verification_only();
    Ok(secp.verify_ecdsa(&message, &signature, &public_key).is_ok())
}

/// Verify a signed JSON response from the enclave
///
/// This is the main function you'll use to verify enclave responses.
///
/// Parameters:
/// - `json_data`: The JSON data from the response
/// - `signature_hex`: The signature from the response
/// - `public_key_hex`: The enclave's public key (from attestation or initial response)
///
/// Returns: true if signature is valid, false otherwise
pub fn verify_json_signature(
    json_data: &Value,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Canonicalize JSON (same way the enclave does it)
    let canonical_json = canonicalize_json(json_data)?;

    // Verify the signature
    verify_signature(&canonical_json, signature_hex, public_key_hex)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ORBS TEE Nitro - Signature Verification Example\n");

    // Example 1: Verify a simple message signature
    println!("Example 1: Verifying a simple message");
    println!("======================================");

    let message = b"Hello, Enclave!";
    let signature = "3045022100abcd..."; // Example signature (replace with real one)
    let public_key = "0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";

    println!("Message: {:?}", String::from_utf8_lossy(message));
    println!("Signature: {}", signature);
    println!("Public Key: {}", public_key);

    // Note: This will fail because it's an example signature
    match verify_signature(message, signature, public_key) {
        Ok(valid) => println!("Signature valid: {}\n", valid),
        Err(e) => println!("Error verifying: {} (expected - using example data)\n", e),
    }

    // Example 2: Verify a JSON response from the enclave
    println!("Example 2: Verifying a JSON response");
    println!("====================================");

    // Simulate a response from the enclave
    let json_response = serde_json::json!({
        "symbol": "BTCUSDT",
        "price": "42000.50",
        "timestamp": 1234567890
    });

    let response_signature = "abcd1234..."; // Replace with real signature from enclave
    let enclave_public_key = public_key; // Same public key as above

    println!("Response data: {}", serde_json::to_string_pretty(&json_response)?);
    println!("Response signature: {}", response_signature);
    println!("Enclave public key: {}", enclave_public_key);

    match verify_json_signature(&json_response, response_signature, enclave_public_key) {
        Ok(valid) => println!("\nSignature valid: {}", valid),
        Err(e) => println!("\nError verifying: {} (expected - using example data)", e),
    }

    println!("\n\nHow to use this in your application:");
    println!("====================================");
    println!("1. Get the enclave's public key from the attestation document");
    println!("2. Send requests to the enclave via vsocket");
    println!("3. For each signed response:");
    println!("   - Extract the 'data' and 'signature' fields");
    println!("   - Call verify_json_signature(data, signature, public_key)");
    println!("   - Only trust the response if verification succeeds");
    println!("\nExample code:");
    println!("-------------");
    println!(r#"
    // Parse the response
    let response: TeeResponse = serde_json::from_slice(&response_bytes)?;

    // Verify the signature if present
    if let (Some(data), Some(signature)) = (&response.data, &response.signature) {{
        let valid = verify_json_signature(data, signature, &enclave_public_key)?;
        if !valid {{
            return Err("Invalid signature - response may have been tampered with");
        }}
        // Signature is valid - safe to use the data
        println!("Data: {{}}", data);
    }}
"#);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonicalize_json() {
        // Test that JSON objects with same keys in different order produce same output
        let json1 = serde_json::json!({"b": 2, "a": 1});
        let json2 = serde_json::json!({"a": 1, "b": 2});

        let canonical1 = canonicalize_json(&json1).unwrap();
        let canonical2 = canonicalize_json(&json2).unwrap();

        assert_eq!(canonical1, canonical2);
        assert_eq!(canonical1, b"{\"a\":1,\"b\":2}");
    }

    #[test]
    fn test_nested_json_canonicalization() {
        let json = serde_json::json!({
            "outer": {
                "z": 3,
                "a": 1
            },
            "array": [{"b": 2}, {"a": 1}]
        });

        let canonical = canonicalize_json(&json).unwrap();
        let expected = b"{\"array\":[{\"b\":2},{\"a\":1}],\"outer\":{\"a\":1,\"z\":3}}";

        assert_eq!(canonical, expected);
    }

    #[test]
    fn test_verify_signature_invalid_length() {
        let data = b"test data";
        let invalid_sig = "abcd"; // Too short
        let public_key = "0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";

        let result = verify_signature(data, invalid_sig, public_key);
        assert!(result.is_err());
    }
}
