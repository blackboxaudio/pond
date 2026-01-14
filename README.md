# Pond

An interactive ambisonic sound installation where audience members walk through a physical space and tap their phones to create water droplet sounds at their tap location.

## Architecture

```
Mobile Phones (Svelte PWA)
        │
        │ WebSocket (JSON)
        ▼
┌─────────────────────────┐
│  pond-server (Rust)     │
│  ├─ WebSocket Server    │
│  ├─ DSP Graph (bbx_dsp) │
│  └─ Audio Output        │
└───────────┬─────────────┘
            │
            ▼
   Ambisonic Speakers
```

## Project Structure

```
pond/
├── pond-server/    # Rust backend (WebSocket server + DSP)
└── pond-web/       # Svelte 5 frontend (mobile PWA)
```

## Quick Start

### Prerequisites

- Rust (edition 2021)
- Node.js + Yarn
- [bbx_audio](https://github.com/your-org/bbx_audio) workspace at `../../bbx_audio/`

### Running

1. Start the server:
   ```bash
   cd pond-server
   cargo run
   ```

2. Start the web app:
   ```bash
   cd pond-web
   yarn install
   yarn dev
   ```

3. Open the web app on a mobile device connected to the same network

See the individual README files in each subproject for more details.

## Tech Stack

- **Backend**: Rust, bbx_dsp, bbx_net, rodio, tokio
- **Frontend**: Svelte 5, TypeScript, Vite, @bbx-audio/net
