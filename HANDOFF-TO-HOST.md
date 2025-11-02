# Handoff Document: Building the Host Application

This document provides context for building the host application that communicates with this enclave SDK.

## Enclave SDK Location
`/Users/ami/orbs/orbs-tee-enclave-nitro`

## Protocol
Uses `orbs-tee-protocol` v0.1.4 from crates.io

### Request Format (TeeRequest)
```rust
pub struct TeeRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub timestamp: i64,
}
```

### Response Format (TeeResponse)
```rust
pub struct TeeResponse {
    pub id: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub signature: Option<String>,  // Hex-encoded ECDSA signature
    pub error: Option<String>,
}
```

## Communication

### Enclave Side (already implemented)
- Listens on **vsocket** (AWS Nitro specific)
- Default CID: VMADDR_CID_ANY
- Port: Configurable (suggest 3000)
- Wire protocol: 4-byte length prefix + JSON message

### Host Side (to be implemented)
- Connect to enclave via vsocket
- Send TeeRequest as JSON
- Receive TeeResponse as JSON
- Verify signature if present

## Signature Verification

**Algorithm**: ECDSA with secp256k1 curve + SHA-256

**Reference implementation**:
`/Users/ami/orbs/orbs-tee-enclave-nitro/examples/signature-verification/`

Contains:
- Rust verification (src/main.rs)
- JavaScript verification (verify.js)
- TypeScript verification (verify.ts)

**Key points**:
- Signatures are over the `data` field (after JSON canonicalization)
- JSON canonicalization: sorted keys, compact format
- Public key comes from attestation document

## Example Enclave App

**Location**: `examples/price-oracle/`

Example methods:
- `get_price` - Fetch crypto price from Binance
- Returns signed response

## Host Application Architecture (suggested)

```
Host Application
├── vsocket_client.rs    - Connect to enclave via vsock
├── api_server.rs        - HTTP/gRPC API for external clients
├── signature_verify.rs  - Verify enclave responses
├── config.rs           - Configuration (enclave port, etc.)
└── main.rs             - Bootstrap and run
```

## Testing Strategy (from TODO.md Phase 2)

1. **Phase 1**: Build host + guardian repos
2. **Phase 2**: Test on **regular backend** (NOT Nitro yet)
   - Use Unix sockets instead of vsock for local testing
   - Mock attestation for testing
3. **Phase 3**: Move to AWS Nitro Enclaves
   - Test full vsock communication
   - Test real attestation

## Vsocket Protocol Details

**Wire format**:
```
[4 bytes: message length (big-endian u32)]
[N bytes: JSON message]
```

**Reading**:
1. Read 4 bytes for length
2. Read N bytes for message
3. Deserialize JSON

**Writing**:
1. Serialize to JSON
2. Get length as u32
3. Write length (4 bytes big-endian)
4. Write message bytes

## Dependencies for Host

Suggested Cargo.toml dependencies:
```toml
[dependencies]
orbs-tee-protocol = "0.1.4"
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
vsock = "0.3"  # Linux only
secp256k1 = { version = "0.29", features = ["recovery"] }
sha2 = "0.10"
hex = "0.4"

# For API server
axum = "0.7"  # or actix-web, warp, etc.
```

## Key Files to Read

1. `/Users/ami/orbs/orbs-tee-enclave-nitro/README.md` - Full SDK documentation
2. `/Users/ami/orbs/orbs-tee-enclave-nitro/src/vsock.rs` - Vsocket implementation reference
3. `/Users/ami/orbs/orbs-tee-protocol/rust/src/lib.rs` - Protocol types
4. `/Users/ami/orbs/orbs-tee-enclave-nitro/examples/signature-verification/` - Verification examples

## Questions to Ask the New Claude Session

When starting the host repo, say:

> "I'm building a host application that communicates with an AWS Nitro Enclave.
>
> The enclave SDK is at `/Users/ami/orbs/orbs-tee-enclave-nitro`.
>
> Please read:
> 1. `/Users/ami/orbs/orbs-tee-enclave-nitro/README.md`
> 2. `/Users/ami/orbs/orbs-tee-enclave-nitro/HANDOFF-TO-HOST.md` (this file)
>
> Then help me design the host application architecture and implement:
> 1. Vsocket client to connect to enclave
> 2. HTTP API for external clients
> 3. Signature verification
> 4. Request routing"

## Status of Enclave SDK

✅ **Complete**:
- Core SDK implementation
- 25 tests (all passing)
- CI/CD (GitHub Actions)
- Signature verification examples
- Documentation

⏳ **Not yet done**:
- Not published to crates.io (still using git dependency)
- Guardian repo (attestation verification)

## Contact Info

This handoff document was created after completing Task #7 (Signature Verification Examples).
All CI checks pass. SDK is ready for host integration.
