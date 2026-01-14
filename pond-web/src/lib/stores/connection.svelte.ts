/**
 * Connection store - manages WebSocket connection to pond-server.
 */
import { BbxClient } from '@bbx-audio/net'

function getWebSocketUrl(): string {
    if (import.meta.env.VITE_WS_URL) {
        return import.meta.env.VITE_WS_URL
    }
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    return `${protocol}//${window.location.hostname}:8080`
}

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'reconnecting'

let connectionState = $state<ConnectionState>('disconnected')
let client: BbxClient | null = $state(null)
let error = $state<string | null>(null)

async function connect(roomCode: string): Promise<void> {
    if (client) {
        client.disconnect()
    }

    error = null
    connectionState = 'connecting'

    client = new BbxClient({
        url: getWebSocketUrl(),
        roomCode: roomCode.toUpperCase(),
        clientName: 'pond-visitor',
        reconnect: true,
    })

    client.on('connected', () => {
        connectionState = 'connected'
        error = null
    })

    client.on('disconnected', (reason) => {
        connectionState = 'disconnected'
        if (reason) {
            error = reason
        }
    })

    client.on('reconnecting', () => {
        connectionState = 'reconnecting'
    })

    client.on('error', (err) => {
        error = err.message ?? 'Connection error'
    })

    try {
        await client.connect()
    } catch (e) {
        connectionState = 'disconnected'
        error = e instanceof Error ? e.message : 'Failed to connect'
        client = null
        throw e
    }
}

function disconnect(): void {
    client?.disconnect()
    client = null
    connectionState = 'disconnected'
}

function trigger(name: string): void {
    client?.trigger(name)
}

export function getConnectionState() {
    return connectionState
}

export function getClient() {
    return client
}

export function getError() {
    return error
}

export { connect, disconnect, trigger }
