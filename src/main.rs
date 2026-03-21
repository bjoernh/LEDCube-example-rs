mod app;
mod network;
mod protocol;

use std::error::Error;
use network::MatrixConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting LEDCube Rust Client...");
    
    // Default simulator address is commonly 2017, but can be overridden by arguments
    let addr = std::env::args().nth(1).unwrap_or_else(|| "127.0.0.1:2017".to_string());
    println!("Connecting to matrixserver at {}", addr);
    
    let stream = MatrixConnection::connect(&addr).await;
    
    match stream {
        Ok(conn) => {
            println!("Connected successfully!");
            app::run(conn).await?;
        }
        Err(e) => eprintln!("Failed to connect: {}", e),
    }

    Ok(())
}
