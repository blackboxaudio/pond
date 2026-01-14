<script lang="ts">
    import { connect, getConnectionState, getError } from '$lib/stores/connection.svelte'

    let roomCode = $state('')
    let isSubmitting = $state(false)

    const connectionState = $derived(getConnectionState())
    const error = $derived(getError())

    async function handleSubmit(event: SubmitEvent) {
        event.preventDefault()

        if (!roomCode.trim() || isSubmitting) return

        isSubmitting = true

        try {
            await connect(roomCode.trim())
        } catch {
            // Error is handled by the store
        } finally {
            isSubmitting = false
        }
    }
</script>

<div class="room-entry">
    <div class="container">
        <h1>Pond</h1>
        <p class="subtitle">Enter the room code to join</p>

        <form onsubmit={handleSubmit}>
            <input
                type="text"
                bind:value={roomCode}
                placeholder="Room code"
                maxlength="6"
                autocapitalize="characters"
                autocomplete="off"
                disabled={isSubmitting || connectionState === 'connecting'}
            />

            <button
                type="submit"
                disabled={!roomCode.trim() || isSubmitting || connectionState === 'connecting'}
            >
                {#if connectionState === 'connecting'}
                    Connecting...
                {:else}
                    Join
                {/if}
            </button>
        </form>

        {#if error}
            <p class="error">{error}</p>
        {/if}
    </div>
</div>

<style>
    .room-entry {
        display: flex;
        align-items: center;
        justify-content: center;
        min-height: 100dvh;
        padding: 1rem;
        background: linear-gradient(135deg, #0a1628 0%, #1a3a5c 50%, #0d2137 100%);
    }

    .container {
        text-align: center;
        width: 100%;
        max-width: 320px;
    }

    h1 {
        font-size: 2.5rem;
        font-weight: 300;
        color: #7dd3fc;
        margin: 0 0 0.5rem;
        letter-spacing: 0.2em;
    }

    .subtitle {
        color: #64748b;
        margin: 0 0 2rem;
        font-size: 0.875rem;
    }

    form {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    input {
        padding: 1rem;
        font-size: 1.5rem;
        text-align: center;
        letter-spacing: 0.3em;
        text-transform: uppercase;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(125, 211, 252, 0.2);
        border-radius: 0.5rem;
        color: #e2e8f0;
        outline: none;
        transition: border-color 0.2s, background 0.2s;
    }

    input::placeholder {
        color: #475569;
        text-transform: none;
        letter-spacing: normal;
    }

    input:focus {
        border-color: rgba(125, 211, 252, 0.5);
        background: rgba(255, 255, 255, 0.08);
    }

    input:disabled {
        opacity: 0.5;
    }

    button {
        padding: 1rem;
        font-size: 1rem;
        font-weight: 500;
        background: linear-gradient(135deg, #0ea5e9, #0284c7);
        border: none;
        border-radius: 0.5rem;
        color: white;
        cursor: pointer;
        transition: opacity 0.2s, transform 0.1s;
    }

    button:hover:not(:disabled) {
        opacity: 0.9;
    }

    button:active:not(:disabled) {
        transform: scale(0.98);
    }

    button:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .error {
        margin-top: 1rem;
        padding: 0.75rem;
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.3);
        border-radius: 0.5rem;
        color: #fca5a5;
        font-size: 0.875rem;
    }
</style>
