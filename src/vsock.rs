// This file implements the vsocket server that listens for requests from the host
// vsocket is a special socket type for VM-to-host communication

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io;

// vsocket constants
// VMADDR_CID_ANY means "accept connections from any CID (Context ID)"
// CID is like an IP address for VMs
pub const VMADDR_CID_ANY: u32 = 0xFFFFFFFF;

/// vsocket server that accepts connections from the host
/// This is similar to a TCP server but uses vsocket instead
pub struct VsockServer {
    port: u32,
}

impl VsockServer {
    /// Create a new vsocket server on the specified port
    /// 
    /// Common ports:
    /// - 5000: Default for ORBS TEE framework
    /// - The host will connect to "CID 16" (enclave) on this port
    pub fn new(port: u32) -> Self {
        Self { port }
    }
    
    /// Start listening for connections
    /// This is an async function that runs forever, accepting connections
    /// 
    /// For each connection:
    /// 1. Accept the connection
    /// 2. Read the request
    /// 3. Call the handler
    /// 4. Write the response
    /// 5. Close the connection
    pub async fn listen<F, Fut>(
        &self,
        handler: F,
    ) -> Result<(), io::Error>
    where
        // F is a function that takes a request and returns a response
        // Fut is the Future returned by that function (async result)
        F: Fn(Vec<u8>) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Vec<u8>> + Send,
    {
        // Create vsocket listener
        // Note: In actual AWS Nitro, we'd use vsock crate's VsockListener
        // For simplicity, this example shows the structure
        // Real implementation: let listener = vsock::VsockListener::bind(VMADDR_CID_ANY, self.port)?;
        
        // For this design doc, we'll show the pattern:
        println!("Vsock server listening on port {}", self.port);
        
        // Accept connections in a loop
        loop {
            // Accept a connection (blocks until connection arrives)
            // This would be: let (mut stream, _addr) = listener.accept().await?;
            // For now, we'll show the handler pattern
            
            // Clone the handler for this connection (each connection runs in its own task)
            let handler = handler.clone();
            
            // Spawn a new task to handle this connection
            // This allows the server to handle multiple connections concurrently
            tokio::spawn(async move {
                // In real implementation:
                // - Read request from stream
                // - Call handler
                // - Write response to stream
                // - Close stream
                
                // Pseudocode:
                // let mut buffer = vec![0u8; 4096];
                // let n = stream.read(&mut buffer).await?;
                // let request = &buffer[..n];
                // let response = handler(request.to_vec()).await;
                // stream.write_all(&response).await?;
            });
        }
    }
}

/// Helper to read a complete message from a vsocket stream
/// Messages are prefixed with a 4-byte length (big-endian)
/// 
/// Wire format:
/// [4 bytes: length][N bytes: data]
pub async fn read_message(stream: &mut TcpStream) -> Result<Vec<u8>, io::Error> {
    // Read 4-byte length prefix
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    
    // Read the actual message
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer).await?;
    
    Ok(buffer)
}

/// Helper to write a complete message to a vsocket stream
/// Prepends a 4-byte length prefix
pub async fn write_message(stream: &mut TcpStream, data: &[u8]) -> Result<(), io::Error> {
    // Write 4-byte length prefix
    let len = data.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    
    // Write the actual message
    stream.write_all(data).await?;
    
    Ok(())
}
