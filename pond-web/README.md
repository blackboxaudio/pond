# pond-web

Svelte 5 mobile PWA for the Pond installation. Provides a full-screen touch surface that sends tap coordinates to the server.

## Development

```bash
yarn install    # Install dependencies
yarn dev        # Start dev server (Vite)
yarn build      # Production build
yarn check      # TypeScript + Svelte type checking
```

## Environment Variables

Create a `.env` file:

```env
VITE_WS_URL=ws://192.168.1.100:8080
```

Replace the IP with your server's address on the local network.

## Features

- **Full-screen touch surface**: Tap anywhere to trigger droplet sounds
- **Multi-touch support**: Each finger creates a separate droplet
- **Ripple animations**: Visual feedback on tap
- **Auto-reconnect**: Automatically reconnects if connection drops
- **Connection indicator**: Shows connection status

## How It Works

1. User opens the PWA on their phone
2. Enters the room code to connect
3. Taps anywhere on the pond surface
4. Touch coordinates are normalized (0-1) and sent via WebSocket
5. Server synthesizes a spatialized droplet sound at that position
