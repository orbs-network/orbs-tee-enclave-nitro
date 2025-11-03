# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

This is the **ORBS TEE Nitro SDK** - a Rust framework for building AWS Nitro Enclave applications. The SDK provides a high-level abstraction for developers to build Trusted Execution Environment (TEE) applications without dealing with low-level Nitro APIs.

**Key insight**: This SDK acts as a runtime that orchestrates cryptographic operations, attestation, and communication, allowing developers to focus solely on their business logic.

**Integration Testing**: See `/home/ubuntu/INTEGRATION_TESTING.md` for complete guide on testing enclave + host communication.

## Build and Test Commands

### Development (macOS/Linux without Nitro)
```bash
# Build the library (without nitro features)
cargo build --no-default-features

# Run all tests (cross-platform, 16 tests)
cargo test --no-default-features

# Run specific test file
cargo test --no-default-features --test crypto_tests
cargo test --no-default-features --test runtime_tests

# Check code without building
cargo check --no-default-features

# Format code
cargo fmt

# Run clippy linter
cargo clippy --no-default-features
```

### Production (AWS Nitro Enclaves on Linux)
```bash
# Build with nitro features (requires Linux)
cargo build --release

# Run all tests with nitro features
cargo test

# Build the price-oracle example (requires Linux)
cargo build --release --manifest-path examples/price-oracle/Cargo.toml
```

### Minimum Supported Rust Version (MSRV)
- **Rust 1.83+** required (due to dependencies like reqwest, icu_collections)
- Update Rust: `rustup update stable`

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

6. **Protocol** (from `orbs-tee-protocol` crate) - Request/response data structures
   - Published on crates.io as `orbs-tee-protocol = "0.1.4"`
   - Shared between Rust (enclave) and TypeScript (host) implementations
   - `TeeRequest`: Contains id, method, params, timestamp
   - `TeeResponse`: Contains id, success, data, signature, error
   - Re-exported from crate root: `pub use orbs_tee_protocol::{TeeRequest, TeeResponse}`

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

## Test Structure

### Directory Layout
```
orbs-tee-enclave-nitro/
├── src/
│   ├── lib.rs          # Public API, EnclaveApp trait
│   ├── app.rs          # EnclaveRuntime implementation (165 lines)
│   ├── crypto.rs       # KeyManager with ECDSA signing
│   ├── nitro.rs        # NitroAttestation (Linux-only)
│   └── vsock.rs        # VsockServer (Linux-only)
├── tests/
│   ├── crypto_tests.rs    # 10 integration tests for crypto module
│   └── runtime_tests.rs   # 6 integration tests for EnclaveApp/runtime
├── examples/
│   └── price-oracle/      # Complete example application
└── TODO.md                # Development task tracking
```

### Test Coverage (16 tests total)

**Crypto Tests** (`tests/crypto_tests.rs` - 10 tests)
- Key generation and validation
- Public key format (bytes and hex)
- Data signing and JSON signing
- Signature verification with secp256k1
- Deterministic signing behavior
- Different data produces different signatures
- Key manager cloning
- Unique keys per instance

**Runtime Tests** (`tests/runtime_tests.rs` - 6 tests)
- Application initialization lifecycle
- Request handling (echo, signed echo)
- Error handling (internal errors, invalid methods)
- Error message formatting
- EnclaveApp trait implementation

### Running Tests

```bash
# Run all tests (cross-platform)
cargo test --no-default-features

# Run with verbose output
cargo test --no-default-features -- --nocapture

# Run specific test
cargo test --no-default-features test_signature_verification

# Run tests in specific file
cargo test --no-default-features --test crypto_tests
```

### Cross-Platform Testing

**macOS/Windows (Development)**
- ✅ All 16 tests pass with `--no-default-features`
- ✅ Tests crypto module thoroughly
- ✅ Tests runtime request/response handling
- ❌ Cannot test nitro/vsock (Linux-only)

**Linux (Production)**
- ✅ All tests including nitro features
- ✅ vsock and NSM device available
- ✅ Full integration testing possible

### Feature Flags

```toml
[features]
default = ["nitro"]
nitro = ["aws-nitro-enclaves-nsm-api", "vsock"]
```

- **Default**: Includes nitro features (for AWS Nitro Enclaves on Linux)
- **No defaults**: Cross-platform development and testing

## Development Notes

### Working with Nitro-Specific Code

- The `NitroAttestation` module uses unsafe FFI calls to the NSM device
- NSM file descriptor must be properly closed (handled in `Drop` implementation)
- Attestation documents are CBOR-encoded and contain AWS certificate chains

### Testing Considerations

**Platform Support:**
- ✅ **Cross-platform tests**: Run on macOS/Windows/Linux with `--no-default-features`
- ✅ **16 integration tests** covering crypto and runtime
- ⚠️ **Nitro-specific code** (NSM device) only works inside AWS Nitro Enclaves on Linux
- ⚠️ **vsocket** requires Linux environment

**Test Organization:**
- Unit tests removed from `src/*.rs` files
- All tests moved to `tests/` directory as integration tests
- Integration tests compile as separate crates, testing public API
- Better separation and organization

**Conditional Compilation:**
- Use `#[cfg(feature = "nitro")]` for platform-specific code
- Runtime has two `new()` implementations: one with nitro, one without
- Tests use the no-feature version for cross-platform compatibility

### Cryptographic Guarantees

- Private keys are generated using `OsRng` (cryptographically secure)
- ECDSA signatures use secp256k1 curve (same as Bitcoin/Ethereum)
- All data is hashed with SHA-256 before signing
- Signatures are 64 bytes (32-byte r component + 32-byte s component)

## Dependencies

**Core Dependencies (always included):**
- `orbs-tee-protocol = "0.1.4"`: Shared protocol types (TeeRequest/TeeResponse)
- `secp256k1`: ECDSA signing and key generation (secp256k1 curve)
- `sha2`: SHA-256 hashing for signatures
- `tokio`: Async runtime for non-blocking I/O
- `serde`/`serde_json`: JSON serialization/deserialization
- `async-trait`: Async methods in traits
- `thiserror`: Better error types
- `hex`: Hex encoding/decoding
- `rand`: Cryptographically secure random number generation
- `chrono`: Date/time handling

**Optional Dependencies (nitro feature):**
- `aws-nitro-enclaves-nsm-api`: Low-level NSM device communication (Linux-only)
- `vsock`: vsocket communication protocol (Linux-only)
- `libc`: For closing file descriptors

**Development Strategy:**
- Using local path for examples: `orbs-tee-nitro = { path = "../../" }`
- Will publish to crates.io after completing quality checklist (see TODO.md)
- Following semantic versioning (0.1.0 for first release)
