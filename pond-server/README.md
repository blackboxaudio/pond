# pond-server

Rust backend for the Pond installation. Receives touch events via WebSocket and synthesizes spatialized water droplet sounds.

## Development

```bash
cargo build     # Build the server
cargo run       # Run the server (binds to 0.0.0.0:8080)
cargo check     # Type check without building
cargo clippy    # Run linter
```

## Configuration

| Setting      | Default            |
|--------------|--------------------|
| Bind address | `0.0.0.0:8080`     |
| Room code    | `POND01`           |
| Sample rate  | 48000 Hz           |
| Buffer size  | 512 samples        |
| Voice count  | 8 polyphonic       |

## Dependencies

This project depends on the `bbx_audio` workspace for DSP and networking:

```toml
bbx_dsp = { path = "../../bbx_audio/bbx_dsp" }
bbx_net = { path = "../../bbx_audio/bbx_net", features = ["websocket"] }
```

Ensure the `bbx_audio` repository is cloned at the expected relative path.

## Message Protocol

Touch triggers are received as:
```
droplet:x,y
```

Where `x` and `y` are normalized coordinates (0-1). X maps to pan position, Y affects pitch.
