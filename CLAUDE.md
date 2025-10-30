# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is the **ORBS TEE Nitro SDK** - a Rust framework for building AWS Nitro Enclave applications. The SDK provides a high-level abstraction for developers to build Trusted Execution Environment (TEE) applications without dealing with low-level Nitro APIs.

**Key insight**: This SDK acts as a runtime that orchestrates cryptographic operations, attestation, and communication, allowing developers to focus solely on their business logic.

## Build and Test Commands

```bash
# Build the library
cargo build

# Build with release optimizations
cargo build --release

# Run tests
cargo test

# Build the price-oracle example
cargo build --manifest-path examples/price-oracle/Cargo.toml

# Build price-oracle with release optimizations
cargo build --release --manifest-path examples/price-oracle/Cargo.toml

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy
```

## Architecture

### Core Components

The SDK is structured around several key modules that work together:

1. **EnclaveRuntime** (`src/app.rs`) - The orchestrator that brings everything together
   - Creates and manages the key pair
   - Initializes Nitro attestation
   - Starts the vsocket server
   - Routes requests to user applications
   - Automatically signs responses when requested

2. **EnclaveApp Trait** (`src/lib.rs`) - The interface developers implement
   - `init()` - Called once at enclave startup for initialization
   - `handle_request()` - Processes custom method calls with business logic
   - Developers implement this trait and the framework handles everything else

3. **KeyManager** (`src/crypto.rs`) - Cryptographic operations
   - Generates a single ECDSA key pair per enclave instance
   - Private key NEVER leaves the enclave
   - Provides signing functionality for data and JSON
   - Uses secp256k1 curve with SHA-256 hashing

4. **NitroAttestation** (`src/nitro.rs`) - AWS NSM hardware interface
   - Communicates with `/dev/nsm` device
   - Generates attestation documents containing PCRs, public key, and AWS certificate chain
   - Attestation proves the enclave's code and embedded public key

5. **VsockServer** (`src/vsock.rs`) - Communication layer
   - Listens for connections from the host VM
   - Uses vsocket (VM-to-host communication protocol)
   - Messages use length-prefixed wire format: `[4 bytes: length][N bytes: data]`

6. **Protocol** (`src/protocol.rs`) - Request/response data structures
   - `TeeRequest`: Contains id, method, params, timestamp
   - `TeeResponse`: Contains id, success, data, signature, error

### Request Flow

```
Host → vsocket → EnclaveRuntime → User's EnclaveApp → handle_request()
                                                          ↓
                                                    Business Logic
                                                          ↓
Host ← vsocket ← EnclaveRuntime ← Response (optionally signed)
```

1. Host sends JSON request over vsocket
2. EnclaveRuntime deserializes and routes to user's `handle_request()`
3. User code implements business logic (e.g., fetch price, process data)
4. User returns `Response { data, sign: bool }`
5. If `sign: true`, runtime signs the response with enclave's private key
6. Runtime serializes and sends response back to host

### Key Design Patterns

- **Separation of Concerns**: Framework handles all TEE infrastructure; developers only write business logic
- **Type Safety**: Strong Rust types prevent entire classes of bugs
- **Async/Await**: All I/O operations are non-blocking using Tokio
- **Generic Runtime**: `EnclaveRuntime<T>` works with any type `T` that implements `EnclaveApp`
- **Single Key Pair**: Each enclave instance has one key pair that persists for the enclave's lifetime

## Example Structure

See `examples/price-oracle/` for a complete working example:
- Implements `EnclaveApp` trait
- Fetches cryptocurrency prices from Binance API
- Returns signed responses
- Only ~30 lines of business logic

The example demonstrates the SDK's core value proposition: developers write minimal code while getting full TEE functionality.

## Development Notes

### Working with Nitro-Specific Code

- The `NitroAttestation` module uses unsafe FFI calls to the NSM device
- NSM file descriptor must be properly closed (handled in `Drop` implementation)
- Attestation documents are CBOR-encoded and contain AWS certificate chains

### Testing Considerations

- Nitro-specific code (NSM device) only works inside actual AWS Nitro Enclaves
- For local development, mock the `NitroAttestation` interface
- vsocket functionality requires VM environment or mocking

### Cryptographic Guarantees

- Private keys are generated using `OsRng` (cryptographically secure)
- ECDSA signatures use secp256k1 curve (same as Bitcoin/Ethereum)
- All data is hashed with SHA-256 before signing
- Signatures are 64 bytes (32-byte r component + 32-byte s component)

## Dependencies

Key external crates:
- `aws-nitro-enclaves-nsm-api`: Low-level NSM device communication
- `secp256k1`: ECDSA signing and key generation
- `tokio`: Async runtime
- `serde`/`serde_json`: Serialization
- `vsock`: vsocket communication protocol
