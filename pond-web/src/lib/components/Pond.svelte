<script lang="ts">
    import { trigger, getConnectionState, disconnect } from '$lib/stores/connection.svelte'

    interface Ripple {
        id: number
        x: number
        y: number
    }

    let ripples = $state<Ripple[]>([])
    let rippleId = 0

    const connectionState = $derived(getConnectionState())

    const TOUCH_DEBOUNCE_MS = 150
    const touchLastTrigger = new Map<number, number>()
    let lastTouchTime = 0

    function spawnRipple(x: number, y: number) {
        const id = rippleId++
        ripples.push({ id, x, y })

        // Remove ripple after animation completes
        setTimeout(() => {
            ripples = ripples.filter((r) => r.id !== id)
        }, 1500)
    }

    function handleTouchStart(event: TouchEvent) {
        event.preventDefault()
        lastTouchTime = performance.now()

        for (const touch of event.changedTouches) {
            const lastTrigger = touchLastTrigger.get(touch.identifier) ?? 0
            if (lastTouchTime - lastTrigger < TOUCH_DEBOUNCE_MS) continue

            touchLastTrigger.set(touch.identifier, lastTouchTime)

            const x = touch.clientX / window.innerWidth
            const y = touch.clientY / window.innerHeight

            trigger(`droplet:${x.toFixed(4)},${y.toFixed(4)}`)
            spawnRipple(touch.clientX, touch.clientY)
        }
    }

    function handleTouchEnd(event: TouchEvent) {
        for (const touch of event.changedTouches) {
            touchLastTrigger.delete(touch.identifier)
        }
    }

    function handleClick(event: MouseEvent) {
        if (performance.now() - lastTouchTime < 500) return

        const x = event.clientX / window.innerWidth
        const y = event.clientY / window.innerHeight

        trigger(`droplet:${x.toFixed(4)},${y.toFixed(4)}`)
        spawnRipple(event.clientX, event.clientY)
    }

    function handleDisconnect() {
        disconnect()
    }

    function handleKeydown(event: KeyboardEvent) {
        if (event.key === ' ' || event.key === 'Enter') {
            event.preventDefault()
            // Trigger at center when using keyboard
            trigger(`droplet:0.5,0.5`)
            spawnRipple(window.innerWidth / 2, window.innerHeight / 2)
        }
    }
</script>

<div
    class="pond"
    role="button"
    tabindex="0"
    ontouchstart={handleTouchStart}
    ontouchend={handleTouchEnd}
    onclick={handleClick}
    onkeydown={handleKeydown}
>
    <!-- Connection status indicator -->
    <div class="status" class:connected={connectionState === 'connected'} class:reconnecting={connectionState === 'reconnecting'}>
        <div class="status-dot"></div>
        {#if connectionState === 'reconnecting'}
            <span class="status-text">Reconnecting...</span>
        {/if}
    </div>

    <!-- Disconnect button -->
    <button class="disconnect" onclick={handleDisconnect}>
        Leave
    </button>

    <!-- Ripple animations -->
    {#each ripples as ripple (ripple.id)}
        <div
            class="ripple"
            style="left: {ripple.x}px; top: {ripple.y}px;"
        ></div>
    {/each}

    <!-- Instructions -->
    <div class="instructions">
        <p>Touch anywhere to create ripples</p>
    </div>
</div>

<style>
    .pond {
        position: fixed;
        inset: 0;
        background: linear-gradient(180deg, #0a1628 0%, #0f2847 30%, #1a4a6e 60%, #0d3555 100%);
        overflow: hidden;
        cursor: pointer;
        touch-action: none;
        user-select: none;
        -webkit-user-select: none;
    }

    .status {
        position: absolute;
        top: 1rem;
        left: 1rem;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        z-index: 10;
    }

    .status-dot {
        width: 12px;
        height: 12px;
        border-radius: 50%;
        background: #ef4444;
        transition: background 0.3s;
    }

    .status.connected .status-dot {
        background: #22c55e;
    }

    .status.reconnecting .status-dot {
        background: #eab308;
        animation: pulse 1s infinite;
    }

    .status-text {
        font-size: 0.75rem;
        color: #94a3b8;
    }

    .disconnect {
        position: absolute;
        top: 1rem;
        right: 1rem;
        padding: 0.5rem 1rem;
        font-size: 0.75rem;
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 0.25rem;
        color: #94a3b8;
        cursor: pointer;
        z-index: 10;
        transition: background 0.2s;
    }

    .disconnect:hover {
        background: rgba(255, 255, 255, 0.15);
    }

    .ripple {
        position: absolute;
        width: 10px;
        height: 10px;
        margin-left: -5px;
        margin-top: -5px;
        border-radius: 50%;
        background: transparent;
        border: 2px solid rgba(125, 211, 252, 0.8);
        animation: ripple-expand 1.5s ease-out forwards;
        pointer-events: none;
    }

    @keyframes ripple-expand {
        0% {
            width: 10px;
            height: 10px;
            margin-left: -5px;
            margin-top: -5px;
            opacity: 1;
            border-width: 2px;
        }
        100% {
            width: 300px;
            height: 300px;
            margin-left: -150px;
            margin-top: -150px;
            opacity: 0;
            border-width: 1px;
        }
    }

    @keyframes pulse {
        0%, 100% {
            opacity: 1;
        }
        50% {
            opacity: 0.5;
        }
    }

    .instructions {
        position: absolute;
        bottom: 2rem;
        left: 0;
        right: 0;
        text-align: center;
        pointer-events: none;
    }

    .instructions p {
        color: rgba(148, 163, 184, 0.5);
        font-size: 0.875rem;
        margin: 0;
    }
</style>
