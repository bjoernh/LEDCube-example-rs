# AGENTS.md - Development Guide for AI Coding Agents

This document provides essential information for AI coding agents working in the LEDCube-example-rs repository.

## Project Overview

A Rust-based async client application for the matrixserver LED Cube framework. Uses Tokio runtime, Protocol Buffers for messaging, COBS framing for TCP communication, and a trait-based animation system with **per-screen animation configuration**.

**Tech Stack**: Rust 2021 Edition, Tokio async runtime, prost (protobuf), COBS encoding

### Key Features
- **Multi-Screen Animation Support**: Different animations on different screens (e.g., fire on front faces, rain on top)
- **Synchronized State Sharing**: Same animation type shares state across screens (synchronized flames)
- **Global Parameters**: Parameter changes affect all instances of an animation type
- **Explicit Opt-In**: Unconfigured screens remain black/off by default

## Build, Test, and Lint Commands

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Clean build artifacts
cargo clean
```

**Note**: The `build.rs` script automatically compiles `proto/matrixserver.proto` into Rust code during builds.

### Running
```bash
# Run with default server (127.0.0.1:2017)
cargo run

# Run with custom server address
cargo run -- 192.168.1.10:44093

# Run release build
cargo run --release
```

### Testing
```bash
# Run all tests (currently none exist)
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test <test_name>

# Run tests in a specific module
cargo test <module_name>::

# Run doc tests only
cargo test --doc
```

**Current State**: No tests exist yet. When adding tests, follow standard Rust conventions with `#[test]` or `#[cfg(test)]` modules.

### Linting and Formatting
```bash
# Check code formatting (doesn't modify files)
cargo fmt -- --check

# Format all code
cargo fmt

# Run clippy linter
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::all

# Run clippy and deny all warnings
cargo clippy -- -D warnings
```

**Note**: No custom rustfmt.toml or clippy.toml exists; use standard Rust defaults.

### Checking
```bash
# Fast compile check without building binaries
cargo check

# Check with all features
cargo check --all-features
```

## Code Style Guidelines

### Naming Conventions
- **Structs/Enums/Traits**: `PascalCase` (e.g., `MatrixConnection`, `AppState`, `Animation`)
- **Functions/Methods**: `snake_case` (e.g., `send_message`, `read_message`, `color_map`)
- **Variables/Parameters**: `snake_case` (e.g., `app_id`, `screen_rotations`, `heat_map`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`, `DEFAULT_PORT`)
- **Modules**: `snake_case` (e.g., `animation`, `network`, `protocol`)
- **Type Parameters**: Single uppercase letter or `PascalCase` (e.g., `T`, `Item`)

### Import Organization
Organize imports in three groups, separated by blank lines:
1. Standard library imports
2. External crate imports
3. Internal crate imports

```rust
// Standard library
use std::time::Duration;
use std::collections::HashMap;
use std::io;

// External crates
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use prost::Message;
use bytes::BytesMut;

// Internal modules
use crate::network::MatrixConnection;
use crate::protocol::matrixserver::{MatrixServerMessage, ServerConfig};
use crate::animation::{AnimationType, AnimationRegistry, Rotation};
use crate::app::ScreenConfig;
```

### Formatting
- **Indentation**: 4 spaces (never tabs)
- **Line Length**: Keep reasonable (80-100 chars preferred, 120 max)
- **Braces**: Opening brace on same line for functions, structs, impls
- **Trailing Commas**: Use in multi-line lists for cleaner diffs
- **Semicolons**: Required for statements, omit for tail expressions

### Types and Type Annotations
- **Explicit Types**: Use when clarity improves or type inference is ambiguous
- **Return Types**: Always specify for public functions
- **Generic Bounds**: Prefer `where` clause for complex bounds
- **Lifetime Elision**: Use when possible; explicit lifetimes when needed

```rust
// Good: clear generic bounds with where clause
pub trait Animation: Send {
    fn update(&mut self, screen: Option<&ScreenInfo>);
    fn render(&self, screen: &ScreenInfo, rotation: Rotation) -> Vec<u8>;
}
```

### Error Handling
- **Prefer `Result<T, E>`**: Never use panic/unwrap in production code paths
- **Use `?` Operator**: Propagate errors upward cleanly
- **Match for Recovery**: Use explicit matching when you can handle specific errors
- **Error Logging**: Use `eprintln!` for error messages (no logging framework currently)
- **Graceful Degradation**: Handle network failures and malformed data gracefully

```rust
// Good: propagate errors with ?
pub async fn send_message(&mut self, msg: &MatrixServerMessage) -> io::Result<()> {
    let framed = encode_message(msg);
    self.stream.write_all(&framed).await?;
    self.stream.flush().await?;
    Ok(())
}

// Good: explicit error handling with logging
match decode_message(&frame) {
    Ok(msg) => return Ok(Some(msg)),
    Err(e) => {
        eprintln!("Failed to decode message: {}", e);
        // Continue processing
    }
}

// Avoid: unwrap in production code (only acceptable in build scripts or with clear justification)
```

### Async Patterns
- **Tokio Runtime**: This is a Tokio-based async application
- **Async Functions**: Mark I/O operations as `async fn`
- **Await**: Use `.await` for async operations
- **Select**: Use `tokio::select!` for multiplexing async operations
- **Send Trait**: Ensure types used across async boundaries implement `Send`

```rust
// Good: async function with proper error handling
pub async fn read_message(&mut self) -> io::Result<Option<MatrixServerMessage>> {
    loop {
        match self.stream.read_u8().await {
            Ok(0) => {
                // Process frame
            }
            Err(e) => return Err(e),
            _ => {}
        }
    }
}
```

### Documentation
- **Public API**: Document all public items with `///` doc comments
- **Module Docs**: Use `//!` at top of module files for module-level documentation
- **Examples**: Include examples in doc comments when helpful
- **Current State**: Minimal documentation exists; add as you develop

```rust
/// Update animation state (called once per frame)
fn update(&mut self, screen: Option<&ScreenInfo>);

/// Render the current state to a screen (returns RGB byte array)
fn render(&self, screen: &ScreenInfo, rotation: Rotation) -> Vec<u8>;
```

## Architecture and Patterns

### Module Structure
- `main.rs` - Entry point, CLI parsing, Tokio runtime setup, animation configuration
- `network.rs` - TCP connection, COBS framing, message I/O
- `protocol.rs` - Protobuf encode/decode with COBS wrapper
- `app.rs` - Application state machine, animation loop coordination, ScreenConfig type
- `animation.rs` - Animation trait, implementations, AnimationType enum, AnimationRegistry
- `build.rs` - Build-time protobuf compilation

### Key Patterns
- **Trait-Based Design**: `Animation` trait for extensible effects
- **State Machine**: `AppState` enum manages application lifecycle
- **Message Passing**: Protocol buffer messages over TCP
- **Builder Pattern**: Used in protobuf generation config
- **Resource Management**: Proper async resource cleanup
- **Registry Pattern**: `AnimationRegistry` manages shared animation instances per type

### Animation Architecture

The system supports different animations on different screens with synchronized state:

```rust
// main.rs - Configure animations per screen
let mut registry = AnimationRegistry::new();
registry.register_fire();  // ONE shared FireAnimation instance

let mut screen_configs: HashMap<i32, ScreenConfig> = HashMap::new();
screen_configs.insert(0, ScreenConfig { animation_type: AnimationType::Fire, rotation: Rotation::Rotate270 });
screen_configs.insert(1, ScreenConfig { animation_type: AnimationType::Fire, rotation: Rotation::Rotate270 });
// Screen 4 not configured → stays black (explicit opt-in)
```

**Key Design Decisions:**
- **Shared State**: One `FireAnimation` instance serves ALL fire screens (synchronized flames)
- **Global Parameters**: Parameter changes affect all instances of an animation type
- **Explicit Opt-In**: Unmapped screens remain black/off by default
- **Active-Only Params**: Only parameters from configured animations exposed to server

### Adding New Animations
1. Create struct implementing `Animation` trait
2. Implement required methods: `update(screen: Option<&ScreenInfo>)`, `render()`
3. Optionally implement `get_schema()` and `handle_param()` for dynamic parameters
4. Add variant to `AnimationType` enum
5. Add registration method to `AnimationRegistry` (e.g., `register_rain()`)
6. Instantiate in `main.rs`: call `registry.register_xxx()` and add to `screen_configs`

## Common Tasks

### Adding Dependencies
Edit `Cargo.toml` under `[dependencies]`, then run `cargo build`

### Modifying Protocol
Edit `proto/matrixserver.proto`, then run `cargo build` to regenerate code

### Performance Targets
- **Frame Rate**: 30 FPS (~33ms per frame)
- **Async I/O**: Non-blocking to maintain frame timing
- **Memory**: Reuse buffers (`BytesMut`) where possible

## Git Commit Guidelines
- Use sentence case with period
- Be descriptive of what changed
- Focus on the "why" when not obvious from the "what"
- Examples: "Introduce an animation trait and a fire animation with rotation support.", "deprecation warning fixed"

## Notes for AI Agents
- **No Tests Yet**: If adding features, consider adding tests
- **Minimal CI/CD**: No automated checks; ensure manual testing
- **Error Handling**: Maintain the robust error handling pattern throughout
- **Async Context**: All I/O should be async; never block the runtime
- **Dead Code**: Use `#[allow(dead_code)]` for intentionally unused code (e.g., example animations)
- **Protocol Changes**: Coordinate with matrixserver repository for proto changes
