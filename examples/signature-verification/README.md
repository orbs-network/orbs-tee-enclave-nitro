# Signature Verification Example

This example demonstrates how to verify cryptographic signatures from ORBS TEE Nitro enclaves. The enclave signs responses with ECDSA (secp256k1 curve) + SHA-256, and you can verify these signatures on the host side to ensure responses haven't been tampered with.

## Overview

When an enclave processes a request and returns a signed response, the signature proves:
1. **Authenticity**: The response came from the enclave (not an imposter)
2. **Integrity**: The response hasn't been modified in transit
3. **Non-repudiation**: The enclave can't deny creating the response

## Signature Format

- **Algorithm**: ECDSA (Elliptic Curve Digital Signature Algorithm)
- **Curve**: secp256k1 (same as Bitcoin/Ethereum)
- **Hash**: SHA-256
- **Signature Size**: 64 bytes (32 bytes r + 32 bytes s)
- **Encoding**: Hex string (128 characters) in JSON responses
- **Public Key Format**: 33 bytes compressed (0x02/0x03 prefix + X coordinate)

## JSON Canonicalization

The enclave uses canonical JSON serialization to ensure deterministic signatures:
1. Object keys are sorted alphabetically
2. No whitespace in the output
3. Consistent serialization across different inputs

**This is critical**: You MUST use the same canonicalization when verifying signatures.

## Examples

### Rust Example

```bash
cd examples/signature-verification
cargo run
```

The Rust example (`src/main.rs`) demonstrates:
- Verifying raw byte signatures
- Verifying JSON response signatures
- JSON canonicalization
- Test cases

Key functions:
```rust
// Verify a signature on raw data
pub fn verify_signature(
    data: &[u8],
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, Box<dyn std::error::Error>>

// Verify a signature on JSON data (with canonicalization)
pub fn verify_json_signature(
    json_data: &Value,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, Box<dyn std::error::Error>>
```

### JavaScript/Node.js Example

```bash
cd examples/signature-verification
npm install
node verify.js
```

The JavaScript example (`verify.js`) demonstrates:
- Verifying signatures in Node.js
- JSON canonicalization
- Using the secp256k1 library

Key functions:
```javascript
// Canonicalize JSON (must match enclave's canonicalization)
function canonicalizeJSON(value)

// Verify a signature on raw data
function verifySignature(data, signatureHex, publicKeyHex)

// Verify a signature on JSON data
function verifyJSONSignature(jsonData, signatureHex, publicKeyHex)
```

### TypeScript Example

```bash
cd examples/signature-verification
npm install
npm run verify-ts
```

The TypeScript example (`verify.ts`) adds type safety:
```typescript
// Type-safe response verification
export function verifyTeeResponse(
    response: TeeResponse,
    publicKeyHex: string
): boolean

// Full type definitions for TEE protocol
export interface TeeResponse {
    id: string;
    success: boolean;
    data?: any;
    signature?: string;
    error?: string;
}
```

## Step-by-Step Verification Process

1. **Get Enclave Public Key**
   - Extract from attestation document
   - Or from initial handshake response
   - Format: 33-byte compressed key as hex string

2. **Receive Signed Response**
   ```json
   {
     "id": "req-123",
     "success": true,
     "data": {
       "symbol": "BTCUSDT",
       "price": "42000.50",
       "timestamp": 1234567890
     },
     "signature": "3045022100abcd...",
     "error": null
   }
   ```

3. **Canonicalize the Data**
   - Sort JSON keys alphabetically
   - Use compact format (no whitespace)
   - Example: `{"price":"42000.50","symbol":"BTCUSDT","timestamp":1234567890}`

4. **Verify the Signature**
   ```rust
   // Rust
   let valid = verify_json_signature(&response.data, &response.signature, &public_key)?;
   ```
   ```javascript
   // JavaScript
   const valid = verifyJSONSignature(response.data, response.signature, publicKey);
   ```

5. **Check Result**
   - `true` → Signature is valid, safe to use data
   - `false` → Signature is invalid, reject the response

## Integration Example

### Rust

```rust
use orbs_tee_protocol::TeeResponse;
use signature_verification::verify_json_signature;

// Parse response
let response: TeeResponse = serde_json::from_slice(&response_bytes)?;

// Verify signature if present
if let (Some(data), Some(signature)) = (&response.data, &response.signature) {
    let valid = verify_json_signature(data, signature, &enclave_public_key)?;

    if !valid {
        return Err("Invalid signature - response may have been tampered with".into());
    }

    // Signature is valid - safe to use the data
    println!("Price: {}", data["price"]);
}
```

### JavaScript/TypeScript

```javascript
const { verifyJSONSignature } = require('./verify');

// Parse response
const response = JSON.parse(responseBytes);

// Verify signature
if (response.data && response.signature) {
    const valid = verifyJSONSignature(
        response.data,
        response.signature,
        enclavePublicKey
    );

    if (!valid) {
        throw new Error('Invalid signature - response may have been tampered with');
    }

    // Signature is valid - safe to use the data
    console.log('Price:', response.data.price);
}
```

## Security Considerations

### ✅ DO

- Always verify signatures before trusting response data
- Store the enclave's public key securely
- Validate the public key against the attestation document
- Use the same JSON canonicalization as the enclave
- Reject responses with invalid signatures

### ❌ DON'T

- Skip signature verification (even in testing)
- Trust responses without valid signatures
- Use different JSON serialization than the enclave
- Accept signatures from untrusted public keys
- Modify response data before verification

## Dependencies

### Rust
- `secp256k1` - ECDSA signature verification
- `sha2` - SHA-256 hashing
- `hex` - Hex encoding/decoding
- `serde_json` - JSON handling

### JavaScript/TypeScript
- `secp256k1` - ECDSA signature verification (npm package)
- `crypto` - SHA-256 hashing (Node.js built-in)
- `@types/secp256k1` - TypeScript type definitions (dev dependency)

## Testing

### Rust
```bash
cargo test
```

Tests include:
- JSON canonicalization consistency
- Nested object canonicalization
- Invalid signature detection

### JavaScript
The JavaScript example includes inline examples that demonstrate usage.

## Troubleshooting

### Signature Verification Fails

**Problem**: `verify_signature()` returns `false`

**Possible Causes**:
1. **Wrong JSON canonicalization** - Ensure keys are sorted and no whitespace
2. **Hex decoding error** - Check that signature/public key are valid hex strings
3. **Modified data** - The data may have been altered after signing
4. **Wrong public key** - Using public key from different enclave instance

**Solutions**:
```rust
// Debug: Print canonical JSON
let canonical = canonicalize_json(&data)?;
println!("Canonical JSON: {}", String::from_utf8(canonical)?);

// Debug: Print signature and public key
println!("Signature: {}", signature_hex);
println!("Public Key: {}", public_key_hex);
```

### Invalid Signature Length

**Problem**: "Invalid signature length: expected 64 bytes"

**Cause**: Signature is not in compact format (64 bytes)

**Solution**: Ensure signature is in compact format, not DER format

### JSON Canonicalization Mismatch

**Problem**: Signature verification fails on valid data

**Cause**: Different JSON canonicalization between enclave and verifier

**Solution**:
- Ensure both use sorted keys
- Ensure both use compact format (no whitespace)
- Test with simple JSON first: `{"a":1,"b":2}`

## Additional Resources

- [secp256k1 crate documentation](https://docs.rs/secp256k1/)
- [ECDSA Wikipedia](https://en.wikipedia.org/wiki/Elliptic_Curve_Digital_Signature_Algorithm)
- [ORBS TEE Protocol](https://github.com/orbs-network/orbs-tee-protocol)

## License

MIT
