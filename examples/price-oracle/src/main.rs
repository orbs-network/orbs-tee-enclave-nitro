// Price Oracle Example - Shows how to use the ORBS TEE Nitro Framework
// This is a complete working example with only ~30 lines of business logic!

use orbs_tee_nitro::{EnclaveApp, run_enclave_app, Response, AppError};
use async_trait::async_trait;
use serde_json::json;

/// Our Price Oracle application
/// This struct can hold any state your app needs
struct PriceOracle {
    // Could add state here like:
    // - API keys
    // - Cached prices
    // - Configuration
}

impl PriceOracle {
    fn new() -> Self {
        Self {}
    }
}

// Implement the EnclaveApp trait
// This is where your business logic goes
#[async_trait]
impl EnclaveApp for PriceOracle {
    /// Called once when enclave starts
    /// Use this to initialize any state
    async fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌟 Price Oracle starting up...");
        println!("📊 Ready to fetch prices from Binance");
        Ok(())
    }

    /// Handle custom requests
    /// This is where you implement your app's methods
    async fn handle_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Response, AppError> {
        match method {
            // Handle "get_price" requests
            "get_price" => {
                // Extract symbol from parameters (default to BTCUSDT)
                let symbol = params.get("symbol")
                    .and_then(|v| v.as_str())
                    .unwrap_or("BTCUSDT");

                println!("💰 Fetching price for {}", symbol);

                // Fetch price from Binance API
                // This is YOUR business logic - the framework doesn't interfere
                let url = format!(
                    "https://api.binance.com/api/v3/ticker/price?symbol={}",
                    symbol
                );
                
                let response = reqwest::get(&url)
                    .await
                    .map_err(|e| AppError::NetworkError(e.to_string()))?;
                
                let price_data: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|e| AppError::NetworkError(e.to_string()))?;

                let price = price_data["price"]
                    .as_str()
                    .ok_or_else(|| AppError::InternalError("No price in response".to_string()))?;

                println!("✓ Price fetched: {} = {}", symbol, price);

                // Return response
                // Set sign=true so framework will sign it with enclave's private key
                Ok(Response {
                    data: json!({
                        "symbol": symbol,
                        "price": price,
                        "timestamp": chrono::Utc::now().timestamp(),
                        "source": "binance"
                    }),
                    sign: true,  // Framework will automatically sign this response
                })
            }
            
            // Unknown method
            _ => Err(AppError::InvalidRequest(
                format!("Unknown method: {}", method)
            )),
        }
    }
}

// Main function - entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("════════════════════════════════════════");
    println!("   ORBS TEE Price Oracle Enclave");
    println!("════════════════════════════════════════");
    println!();

    // Create your app and run it
    // Framework handles everything else!
    run_enclave_app(PriceOracle::new()).await
}
