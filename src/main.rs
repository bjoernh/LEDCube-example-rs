mod animation;
mod app;
mod network;
mod protocol;

use network::MatrixConnection;
use std::collections::HashMap;
use std::error::Error;
use animation::Rotation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting LEDCube Rust Client...");

    // Default simulator address is commonly 2017, but can be overridden by arguments
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:2017".to_string());
    println!("Connecting to matrixserver at {}", addr);

    let stream = MatrixConnection::connect(&addr).await;

    match stream {
        Ok(conn) => {
            println!("Connected successfully!");
            let mut rotations = HashMap::new();
            // Defined based on CubeLayout.ts rotations + 270 degree local rotation
            rotations.insert(0, Rotation::Rotate270); // Front
            rotations.insert(1, Rotation::Rotate270); // Right
            rotations.insert(2, Rotation::Rotate270); // Back
            rotations.insert(3, Rotation::Rotate270); // Left
            
            let anim = Box::new(animation::FireAnimation::new());
            app::run(conn, anim, rotations).await?;
        }
        Err(e) => eprintln!("Failed to connect: {}", e),
    }

    Ok(())
}
