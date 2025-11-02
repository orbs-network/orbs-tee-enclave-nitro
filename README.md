# ORBS TEE Enclave - Nitro

A Rust SDK for building Trusted Execution Environment (TEE) applications on AWS Nitro Enclaves.

## Overview

This SDK provides a high-level framework for building secure enclave applications without dealing with low-level Nitro APIs. It handles all the infrastructure complexity (cryptography, attestation, communication) so you can focus on your business logic.

## Features

- **Simple API**: Implement one trait and you're done
- **Automatic Key Management**: ECDSA key pair generation and signing
- **Built-in Attestation**: AWS Nitro attestation document generation
- **Secure Communication**: vsocket-based host-enclave messaging
- **Type Safety**: Leverages Rust's strong type system
- **Async/Await**: Non-blocking I/O with Tokio

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
orbs-tee-nitro = { git = "https://github.com/orbs-network/orbs-tee-enclave-nitro" }
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Quick Start

Here's a complete working example in ~30 lines:

```rust
use orbs_tee_nitro::{EnclaveApp, run_enclave_app, Response, AppError};
use async_trait::async_trait;
use serde_json::json;

// 1. Define your application struct
struct MyApp;

// 2. Implement the EnclaveApp trait
#[async_trait]
impl EnclaveApp for MyApp {
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("App initialized!");
        Ok(())
    }

    async fn handle_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Response, AppError> {
        match method {
            "hello" => {
                let name = params.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("World");

                Ok(Response {
                    data: json!({
                        "message": format!("Hello, {}!", name)
                    }),
                    sign: true  // SDK will sign this response
                })
            }
            _ => Err(AppError::InvalidRequest(
                format!("Unknown method: {}", method)
            ))
        }
    }
}

// 3. Run your app
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_enclave_app(MyApp).await
}
```

That's it! The framework handles:
- ✅ Generating cryptographic keys
- ✅ Creating Nitro attestation documents
- ✅ Setting up vsocket server
- ✅ Routing requests to your handler
- ✅ Signing responses

## Architecture

The SDK consists of these core components:

### 1. EnclaveApp Trait
The interface you implement with your business logic:

```rust
#[async_trait]
pub trait EnclaveApp: Send + Sync {
    /// Called once at startup
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Handle custom requests
    async fn handle_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Response, AppError>;
}
```

### 2. EnclaveRuntime
The orchestrator that brings everything together:
- Creates and manages the ECDSA key pair
- Initializes AWS Nitro attestation
- Starts the vsocket server
- Routes requests to your app
- Signs responses when requested

### 3. KeyManager
Handles cryptographic operations:
- Generates secp256k1 key pair
- Signs data with ECDSA + SHA-256
- Private key never leaves the enclave

### 4. NitroAttestation
Interfaces with AWS NSM (Nitro Secure Module):
- Generates attestation documents
- Embeds public key in attestation
- Provides proof of code integrity

### 5. VsockServer
Communication layer:
- Listens on vsocket for host connections
- Uses length-prefixed message format
- Handles concurrent connections

### 6. Protocol
Request/response data structures from the shared `orbs-tee-protocol` package:

```rust
pub struct TeeRequest {
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
    pub timestamp: i64,
}

pub struct TeeResponse {
    pub id: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub signature: Option<String>,
    pub error: Option<String>,
}
```

Note: These types are imported from the [`orbs-tee-protocol`](https://github.com/orbs-network/orbs-tee-protocol) package, which provides shared protocol definitions for both Rust (enclave) and TypeScript (host) implementations.

## Complete Example: Price Oracle

See `examples/price-oracle/` for a complete working example that fetches cryptocurrency prices from Binance:

```rust
struct PriceOracle;

#[async_trait]
impl EnclaveApp for PriceOracle {
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Price Oracle starting...");
        Ok(())
    }

    async fn handle_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Response, AppError> {
        match method {
            "get_price" => {
                let symbol = params.get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("BTCUSDT");

                // Fetch from Binance API
                let url = format!(
                    "https://api.binance.com/api/v3/ticker/price?symbol={}",
                    symbol
                );
                let response = reqwest::get(&url).await
                    .map_err(|e| AppError::NetworkError(e.to_string()))?;

                let price_data: serde_json::Value = response.json().await
                    .map_err(|e| AppError::NetworkError(e.to_string()))?;

                Ok(Response {
                    data: json!({
                        "symbol": symbol,
                        "price": price_data["price"],
                        "timestamp": chrono::Utc::now().timestamp(),
                    }),
                    sign: true
                })
            }
            _ => Err(AppError::InvalidRequest(format!("Unknown method: {}", method)))
        }
    }
}
```

## Building and Testing

### Build Commands

```bash
# Build the library
cargo build

# Build with optimizations
cargo build --release

# Build the price oracle example
cargo build --manifest-path examples/price-oracle/Cargo.toml

# Check code
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

### Running Tests

The SDK includes comprehensive test coverage with 16 tests across cryptography and runtime components.

#### Cross-Platform Testing

**macOS/Windows:**
```bash
# Run all tests (without nitro features)
cargo test --no-default-features

# Run tests with verbose output
cargo test --no-default-features -- --nocapture
```

**Linux Only:**
```bash
# Run all tests (including nitro features)
cargo test

# Run specific test suites
cargo test --test crypto_tests
cargo test --test runtime_tests
```

**Why the difference?** The `nitro` feature includes Linux-only dependencies (vsock, NSM) that don't compile on macOS/Windows. Tests are designed to run without these features for cross-platform development, while full nitro tests run on Linux via CI or Docker.

#### Test Structure

The test suite is organized into:

- **Unit Tests** (`tests/crypto_tests.rs` - 10 tests)
  - Key generation and uniqueness
  - Signature creation and verification
  - Public key serialization
  - JSON signing and verification

- **Integration Tests** (`tests/runtime_tests.rs` - 6 tests)
  - EnclaveRuntime initialization
  - Request handling and routing
  - Response signing
  - Error handling

#### Feature Flags

The SDK uses Cargo features for platform-specific code:

```toml
[features]
default = ["nitro"]
nitro = ["aws-nitro-enclaves-nsm-api", "vsock"]
```

- **With nitro** (default on Linux): Full functionality including vsocket and NSM attestation
- **Without nitro** (`--no-default-features`): Core crypto and runtime features for cross-platform testing

#### Docker Testing (Linux Environment)

For testing nitro features on macOS/Windows, use Docker:

```bash
# Run cross-platform tests in Docker
make docker-test

# Test with nitro features enabled
make docker-test-nitro

# Check that nitro features compile
make docker-check

# Run all checks (test, clippy, fmt)
make docker-all

# Open shell in container for debugging
make docker-shell
```

Docker provides a Linux environment where the vsock crate compiles correctly.

#### Continuous Integration

The project uses GitHub Actions for automated testing on every push and pull request:

- ✅ **Tests** on Ubuntu, macOS, and Windows
- ✅ **Clippy lints** (with and without nitro features)
- ✅ **Format check** (cargo fmt)
- ✅ **MSRV check** (Minimum Supported Rust Version: 1.83)
- ✅ **Example compilation** (price-oracle on Linux)

View CI status: [GitHub Actions](https://github.com/orbs-network/orbs-tee-enclave-nitro/actions)

## How It Works

### Request Flow

```
Host Application
    ↓ (sends TeeRequest via vsocket)
VsockServer
    ↓ (deserializes JSON)
EnclaveRuntime
    ↓ (routes to your app)
Your EnclaveApp::handle_request()
    ↓ (your business logic)
Response { data, sign: true }
    ↓ (if sign=true, KeyManager signs)
EnclaveRuntime
    ↓ (serializes to TeeResponse)
VsockServer
    ↓ (sends back via vsocket)
Host Application
```

### Attestation Flow

1. Enclave starts, generates ECDSA key pair
2. Host requests attestation document
3. NitroAttestation embeds public key in document
4. NSM hardware signs document with AWS certificate chain
5. Host verifies attestation proves:
   - Code integrity (via PCRs)
   - Public key belongs to this enclave
   - Document signed by AWS

### Security Guarantees

- **Private Key Never Leaves Enclave**: Generated inside, never exported
- **Code Integrity**: Attestation proves exact code running
- **Tamper Proof**: Any modification invalidates attestation
- **Cryptographically Secure**: Uses OS random number generator
- **Standard Algorithms**: secp256k1 + SHA-256 (same as Bitcoin/Ethereum)

## API Reference

### EnclaveApp Trait

```rust
#[async_trait]
pub trait EnclaveApp: Send + Sync {
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn handle_request(&self, method: &str, params: Value) -> Result<Response, AppError>;
}
```

### Response Type

```rust
pub struct Response {
    pub data: serde_json::Value,  // Your response data
    pub sign: bool,                // Whether to sign with enclave key
}
```

### AppError Type

```rust
pub enum AppError {
    InvalidRequest(String),
    InternalError(String),
    NetworkError(String),
}
```

### Main Entry Point

```rust
pub async fn run_enclave_app<T: EnclaveApp + 'static>(
    app: T
) -> Result<(), Box<dyn std::error::Error>>
```

## Requirements

- **Rust 1.83 or later** (MSRV - Minimum Supported Rust Version)
- **Linux** for nitro features (vsocket, NSM attestation)
- **macOS/Windows** supported for development (without nitro features)
- AWS Nitro Enclave environment (for production deployment)
- Tokio async runtime

## License

MIT

## Contributing

Contributions welcome! Please open an issue or pull request.
