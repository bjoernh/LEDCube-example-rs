#![allow(clippy::enum_variant_names)]

mod animation;
mod app;
mod network;
mod protocol;

use network::MatrixConnection;
use std::collections::HashMap;
use std::error::Error;
use animation::{AnimationType, AnimationRegistry, Rotation};
use app::ScreenConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Starting LEDCube Rust Client...");

    // Default simulator address is commonly 2017, but can be overridden by arguments
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:2017".to_string());
    println!("Connecting to matrixserver at {}", addr);

    // Build animation registry with shared instances
    let mut registry = AnimationRegistry::new();
    registry.register_fire(); // One FireAnimation instance for ALL fire screens

    // Configure which animation goes on each screen
    let mut screen_configs: HashMap<i32, ScreenConfig> = HashMap::new();

    // Front faces - all show synchronized fire
    screen_configs.insert(
        0,
        ScreenConfig {
            animation_type: AnimationType::Fire,
            rotation: Rotation::Rotate270,
        },
    ); // Front
    screen_configs.insert(
        1,
        ScreenConfig {
            animation_type: AnimationType::Fire,
            rotation: Rotation::Rotate270,
        },
    ); // Right
    screen_configs.insert(
        2,
        ScreenConfig {
            animation_type: AnimationType::Fire,
            rotation: Rotation::Rotate270,
        },
    ); // Back
    screen_configs.insert(
        3,
        ScreenConfig {
            animation_type: AnimationType::Fire,
            rotation: Rotation::Rotate270,
        },
    ); // Left

    // Screen 4 (top) - NOT configured = stays black (explicit opt-in)
    // To add later:
    // screen_configs.insert(4, ScreenConfig {
    //     animation_type: AnimationType::Rain,
    //     rotation: Rotation::Rotate0,
    // });

    let stream = MatrixConnection::connect(&addr).await;

    match stream {
        Ok(conn) => {
            println!("Connected successfully!");
            app::run(conn, registry, screen_configs).await?;
        }
        Err(e) => eprintln!("Failed to connect: {}", e),
    }

    Ok(())
}
