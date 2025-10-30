# ORBS TEE Enclave - Nitro

Rust SDK for building AWS Nitro Enclave applications.

## Installation
```toml
[dependencies]
orbs-tee-nitro = "0.1"
```

## Usage
```rust
use orbs_tee_nitro::{EnclaveApp, run_enclave_app, Response, AppError};

struct MyApp;

#[async_trait]
impl EnclaveApp for MyApp {
    async fn handle_request(&self, method: &str, params: Value) -> Result<Response, AppError> {
        // Your business logic here
        Ok(Response { 
            data: json!({"result": "success"}), 
            sign: true 
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_enclave_app(MyApp::new()).await
}
```

## Complete Source Code

For the complete implementation, see the [main design document](computer:///home/claude/orbs-tee-framework-design.md).

Copy these files from the design document:
- `src/lib.rs` (lines 220-338)
- `src/app.rs` (lines 1020-1201) 
- `src/crypto.rs` (lines 619-790)
- `src/nitro.rs` (lines 798-903)
- `src/vsock.rs` (lines 911-1012)
- `examples/price-oracle/src/main.rs` (lines 1456-1562)

## Examples

See `examples/price-oracle/` for a complete working example.

## License

MIT
