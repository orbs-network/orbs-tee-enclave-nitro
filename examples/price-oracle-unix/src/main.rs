/// Price Oracle with Unix Socket Support
/// This is a real price oracle that uses Unix sockets instead of vsocket
/// Perfect for testing on Mac/Linux without Nitro Enclaves

use orbs_tee_protocol::{TeeRequest, TeeResponse};
use orbs_tee_nitro::crypto::KeyManager;
use serde_json::json;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

const SOCKET_PATH: &str = "/tmp/enclave.sock";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("════════════════════════════════════════");
    println!("   ORBS TEE Price Oracle (Unix Socket)");
    println!("════════════════════════════════════════");
    println!();

    // Remove old socket if it exists
    let _ = std::fs::remove_file(SOCKET_PATH);

    // Generate enclave key pair
    println!("🔐 Generating enclave key pair...");
    let key_manager = KeyManager::new()?;
    println!("📝 Public key: {}", key_manager.public_key_hex());
    println!();

    // Create Unix socket listener
    println!("👂 Starting Unix socket server on {}", SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("✅ Ready to accept connections");
    println!("════════════════════════════════════════");
    println!();

    // Accept connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("📞 New connection");
                let km = key_manager.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, km).await {
                        eprintln!("❌ Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("❌ Connection failed: {}", e);
            }
        }
    }

    Ok(())
}

async fn handle_connection(
    mut stream: UnixStream,
    key_manager: KeyManager,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Read length (4 bytes, big-endian)
        let mut length_buf = [0u8; 4];
        match stream.read_exact(&mut length_buf) {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                println!("✓ Client disconnected");
                break;
            },
            Err(e) => return Err(e.into()),
        }

        let length = u32::from_be_bytes(length_buf) as usize;

        // Read message
        let mut message_buf = vec![0u8; length];
        stream.read_exact(&mut message_buf)?;

        // Parse request
        let request: TeeRequest = serde_json::from_slice(&message_buf)?;
        println!("\n→ Request [{}]: {}", request.method, request.id);

        // Handle request
        let response = handle_request(&request, &key_manager).await;

        // Send response
        let response_json = serde_json::to_vec(&response)?;
        let response_length = (response_json.len() as u32).to_be_bytes();

        stream.write_all(&response_length)?;
        stream.write_all(&response_json)?;

        println!("✓ Response [{}]: sent", if response.success { "SUCCESS" } else { "ERROR" });
    }

    Ok(())
}

async fn handle_request(request: &TeeRequest, key_manager: &KeyManager) -> TeeResponse {
    match request.method.as_str() {
        "get_price" => {
            // Extract symbol from parameters
            let symbol = request
                .params
                .get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("BTCUSDT");

            println!("  💰 Fetching price for {} from Binance...", symbol);

            // Fetch from Binance API
            let url = format!(
                "https://api.binance.com/api/v3/ticker/price?symbol={}",
                symbol
            );

            match reqwest::get(&url).await {
                Ok(response) => match response.json::<serde_json::Value>().await {
                    Ok(price_data) => {
                        if let Some(price) = price_data.get("price").and_then(|p| p.as_str()) {
                            println!("  ✓ Price fetched: {} = {}", symbol, price);

                            let data = json!({
                                "symbol": symbol,
                                "price": price,
                                "timestamp": chrono::Utc::now().timestamp(),
                                "source": "binance"
                            });

                            // Sign the response
                            match key_manager.sign_json(&data) {
                                Ok(signature) => {
                                    println!("  ✍️  Response signed");
                                    TeeResponse {
                                        id: request.id.clone(),
                                        success: true,
                                        data: Some(data),
                                        signature: Some(hex::encode(signature)),
                                        error: None,
                                    }
                                }
                                Err(e) => TeeResponse::error(
                                    request.id.clone(),
                                    format!("Signing failed: {}", e),
                                ),
                            }
                        } else {
                            TeeResponse::error(request.id.clone(), "No price in response".to_string())
                        }
                    }
                    Err(e) => TeeResponse::error(
                        request.id.clone(),
                        format!("Failed to parse Binance response: {}", e),
                    ),
                },
                Err(e) => TeeResponse::error(
                    request.id.clone(),
                    format!("Failed to fetch from Binance: {}", e),
                ),
            }
        }

        "get_attestation" => {
            println!("  📜 Generating mock attestation (no real Nitro available)");

            let data = json!({
                "public_key": key_manager.public_key_hex(),
                "note": "Mock attestation - no Nitro NSM device available",
                "timestamp": chrono::Utc::now().timestamp(),
            });

            TeeResponse {
                id: request.id.clone(),
                success: true,
                data: Some(data),
                signature: None,
                error: None,
            }
        }

        _ => TeeResponse::error(
            request.id.clone(),
            format!("Unknown method: {}", request.method),
        ),
    }
}
