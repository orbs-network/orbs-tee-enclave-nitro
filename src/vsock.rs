// This file implements the vsocket server that listens for requests from the host
// vsocket is a special socket type for VM-to-host communication

use std::io::{self, Read, Write};

// Re-export vsock types for convenience
pub use vsock::{VsockAddr, VsockListener, VsockStream, VMADDR_CID_ANY};

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
    ///
    /// Note: This uses blocking I/O from the vsock crate (v0.3) which provides
    /// std::io::Read/Write, not tokio's AsyncRead/AsyncWrite. The accept() loop
    /// runs in a blocking context, but handlers are spawned as async tasks.
    pub async fn listen<F, Fut>(&self, handler: F) -> Result<(), io::Error>
    where
        // F is a function that takes a request and returns a response
        // Fut is the Future returned by that function (async result)
        F: Fn(Vec<u8>) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Vec<u8>> + Send,
    {
        let port = self.port;

        // Get a handle to the current tokio runtime
        // We need this to spawn async tasks from inside the blocking thread
        let runtime_handle = tokio::runtime::Handle::current();

        // Run the blocking accept loop in a separate thread
        // This is necessary because VsockListener::accept() is a blocking operation
        tokio::task::spawn_blocking(move || {
            // Create vsocket listener
            // VMADDR_CID_ANY means accept connections from any Context ID (host or other VMs)
            let addr = VsockAddr::new(VMADDR_CID_ANY, port);
            let listener = VsockListener::bind(&addr)?;

            println!("✅ Vsock server listening on port {}", port);

            // Accept connections in a loop
            loop {
                // Accept a connection (blocks until connection arrives)
                match listener.accept() {
                    Ok((stream, addr)) => {
                        println!("📞 Accepted connection from CID {}", addr.cid());

                        // Clone the handler for this connection
                        let handler = handler.clone();

                        // Spawn async task to handle this connection
                        // Use runtime_handle.spawn() instead of tokio::spawn()
                        // because we're inside a blocking context
                        runtime_handle.spawn(async move {
                            if let Err(e) = handle_connection(stream, handler).await {
                                eprintln!("❌ Error handling connection: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to accept connection: {}", e);
                        // Continue accepting other connections
                    }
                }
            }

            // This code is unreachable because of the infinite loop above
            // The server runs forever until the process is killed
            #[allow(unreachable_code)]
            Ok::<(), io::Error>(())
        })
        .await
        .map_err(io::Error::other)?
    }
}

/// Handle a single connection
/// 1. Read request message
/// 2. Call handler to process request
/// 3. Write response message
/// 4. Close connection (automatic when stream is dropped)
///
/// This function wraps blocking I/O operations in spawn_blocking to work with tokio
async fn handle_connection<F, Fut>(mut stream: VsockStream, handler: F) -> Result<(), io::Error>
where
    F: Fn(Vec<u8>) -> Fut,
    Fut: std::future::Future<Output = Vec<u8>>,
{
    // Read the request message (blocking I/O wrapped in spawn_blocking)
    let request = tokio::task::spawn_blocking(move || {
        let result = read_message(&mut stream);
        (stream, result)
    })
    .await
    .map_err(io::Error::other)?;

    let (mut stream, request) = request;
    let request = request?;

    println!("📥 Received {} bytes", request.len());

    // Process the request (this is already async)
    let response = handler(request).await;

    println!("📤 Sending {} bytes", response.len());

    // Write the response message (blocking I/O wrapped in spawn_blocking)
    tokio::task::spawn_blocking(move || write_message(&mut stream, &response))
        .await
        .map_err(io::Error::other)??;

    // Connection automatically closed when stream is dropped
    Ok(())
}

/// Helper to read a complete message from a vsocket stream
/// Messages are prefixed with a 4-byte length (big-endian)
///
/// Wire format:
/// [4 bytes: length][N bytes: data]
///
/// This uses blocking I/O (std::io::Read)
fn read_message(stream: &mut VsockStream) -> Result<Vec<u8>, io::Error> {
    // Read 4-byte length prefix
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    // Sanity check: prevent allocating huge buffers
    if len > 10 * 1024 * 1024 {
        // 10MB max
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Message too large: {} bytes", len),
        ));
    }

    // Read the actual message
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer)?;

    Ok(buffer)
}

/// Helper to write a complete message to a vsocket stream
/// Prepends a 4-byte length prefix
///
/// This uses blocking I/O (std::io::Write)
fn write_message(stream: &mut VsockStream, data: &[u8]) -> Result<(), io::Error> {
    // Write 4-byte length prefix
    let len = data.len() as u32;
    stream.write_all(&len.to_be_bytes())?;

    // Write the actual message
    stream.write_all(data)?;

    // Flush to ensure data is sent
    stream.flush()?;

    Ok(())
}
