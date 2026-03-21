# LEDCube-example-rs

A **Rust-based Client Application** for the [matrixserver](https://github.com/bjoernh/matrixserver), demonstrating how to connect safely and asynchronously to the LED Cube framework, send protobuf-encoded frames, and interact cleanly without getting dropped by the server.

---

## Features
- **Async & Non-Blocking**: Built on top of [Tokio](https://tokio.rs/), maintaining lightweight, asynchronous I/O and lifecycle loops.
- **Protocol Buffers Integration**: Uses `prost` and `prost-build` within the built-in Cargo `build.rs` to automatically compile `matrixserver.proto` straight into type-safe Rust structs on every build.
- **COBS Message Framing**: A bespoke network reader layer that dynamically unpacks the TCP byte-stream, locates the `0x00` delimiter, and processes the Correct Overhead Byte Stuffing before deserializing.
- **Complete Connection Handshake**: Handles the necessary `RegisterApp` -> `GetServerInfo` signals without provoking server disconnection bugs (i.e. deliberately avoiding superfluous `AppAlive` pings).
- **Dynamic Multi-Screen Rendering**: The client intelligently requests the `ServerConfig`, checks how many screens exist and their individual dimensions (e.g. 6 screens at 64x64), and outputs the exact required geometric payload for every surface.
- **Placeholder Animation**: Ships natively with a 30 FPS spatial RGB color-wipe testing animation.

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

## 🧠 Project Architecture

- **`src/main.rs`**: Establishes the asynchronous Tokio runtime, parses CLI args for the IP/Port payload, and invokes `app::run`.
- **`src/network.rs`**: Manages the `MatrixConnection` object. Implements the buffer loops reading data from the raw TCP stream, finding the COBS zero separators, and safely yielding decoded frames.
- **`src/protocol.rs`**: Contains the `encode_message()` and `decode_message()` helper functions, bridging the COBS framing with the `prost` protobuf deserializer logic.
- **`src/app.rs`**: The heartbeat of the application. Handles the initialization tasks, the select multiplexing for 30 FPS animation generation, and dynamic 3D-array population from user-defined effects.

## 🤝 Next Steps

You can build new animations directly inside `src/app.rs`'s `tokio::select!` interval tick loop. The multi-screen logic ensures your effect propagates across all cube faces effortlessly!
