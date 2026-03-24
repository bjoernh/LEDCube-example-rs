# LEDCube-example-rs

A **Rust-based Client Application** for the [matrixserver](https://github.com/bjoernh/matrixserver), demonstrating how to connect safely and asynchronously to the LED Cube framework, send protobuf-encoded frames, and interact cleanly without getting dropped by the server.

---

## Features
- **Async & Non-Blocking**: Built on top of [Tokio](https://tokio.rs/), maintaining lightweight, asynchronous I/O and lifecycle loops.
- **Protocol Buffers Integration**: Uses `prost` and `prost-build` within the built-in Cargo `build.rs` to automatically compile `matrixserver.proto` straight into type-safe Rust structs on every build.
- **COBS Message Framing**: A bespoke network reader layer that dynamically unpacks the TCP byte-stream, locates the `0x00` delimiter, and processes the Correct Overhead Byte Stuffing before deserializing.
- **Complete Connection Handshake**: Handles the necessary `RegisterApp` -> `GetServerInfo` signals without provoking server disconnection bugs (i.e. deliberately avoiding superfluous `AppAlive` pings).
- **Multi-Screen Animation Support**: Configure different animations per screen with synchronized state sharing across screens using the same animation type.
- **Dynamic Parameter System**: Animations expose adjustable parameters (e.g., fire cooling rate, spark intensity) that can be modified at runtime via the matrixserver UI.
- **Explicit Screen Configuration**: Only configured screens display animations; unconfigured screens remain black/off by default.

## Prerequisites
You need the standard Rust toolchain installed on your matching device.
- [Rust & Cargo](https://rustup.rs/) v1.65+
- (Optional) `Matrixserver Simulator` running locally to verify output.

## Building 

Just run the standard cargo build step. The `build.rs` script will automatically reach over into the `proto/` folder and generate the required modules.

```bash
cargo build --release
```

## Running the Client

By default, the client is programmed to connect to the Simulator on the local loopback `127.0.0.1:2017`. 

```bash
cargo run
```

If your matrixserver is running on a different machine, socket interface, or port configuration, simply pass it as an argument:

```bash
cargo run -- 192.168.1.10:44093
```

## Project Architecture

- **`src/main.rs`**: Establishes the asynchronous Tokio runtime, parses CLI args for the IP/Port payload, configures animation registry and per-screen settings.
- **`src/network.rs`**: Manages the `MatrixConnection` object. Implements the buffer loops reading data from the raw TCP stream, finding the COBS zero separators, and safely yielding decoded frames.
- **`src/protocol.rs`**: Contains the `encode_message()` and `decode_message()` helper functions, bridging the COBS framing with the `prost` protobuf deserializer logic.
- **`src/app.rs`**: The heartbeat of the application. Handles the initialization tasks, the select multiplexing for 30 FPS animation generation, and per-screen rendering coordination.
- **`src/animation.rs`**: Contains the `Animation` trait, `AnimationType` enum, `AnimationRegistry`, and concrete implementations (FireAnimation, DiagonalSweep, SolidColorSweep).

### Animation Configuration Example

```rust
// main.rs - Configure different animations per screen
let mut registry = AnimationRegistry::new();
registry.register_fire();  // Shared instance for all fire screens

let mut screen_configs: HashMap<i32, ScreenConfig> = HashMap::new();
screen_configs.insert(0, ScreenConfig { animation_type: AnimationType::Fire, rotation: Rotation::Rotate270 });
screen_configs.insert(1, ScreenConfig { animation_type: AnimationType::Fire, rotation: Rotation::Rotate270 });
// ... screens 2,3 with fire
// screen 4 (top) not configured → stays black
```

### Key Design Decisions
- **Shared State**: One `FireAnimation` instance serves ALL fire screens (synchronized flames)
- **Global Parameters**: Parameter changes affect all instances of an animation type
- **Explicit Opt-In**: Unconfigured screens remain black/off by default

## Next Steps

### Adding a New Animation (e.g., Rain on Screen 4)

**Step 1**: Implement the animation in `src/animation.rs`:
```rust
pub struct RainAnimation { /* ... */ }
impl Animation for RainAnimation {
    fn update(&mut self, screen: Option<&ScreenInfo>) { /* ... */ }
    fn render(&self, screen: &ScreenInfo, rotation: Rotation) -> Vec<u8> { /* ... */ }
}
```

**Step 2**: Add variant to `AnimationType` enum and registration method:
```rust
pub enum AnimationType { Fire, Rain, /* ... */ }
impl AnimationRegistry {
    pub fn register_rain(&mut self) { /* ... */ }
}
```

**Step 3**: Configure in `src/main.rs`:
```rust
registry.register_rain();
screen_configs.insert(4, ScreenConfig { 
    animation_type: AnimationType::Rain, 
    rotation: Rotation::Rotate0 
});
```

The multi-screen architecture ensures each screen can display different animations while sharing synchronized state when using the same animation type!
