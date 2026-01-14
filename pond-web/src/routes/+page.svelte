<script lang="ts">
    import { getConnectionState } from '$lib/stores/connection.svelte'
    import RoomEntry from '$lib/components/RoomEntry.svelte'
    import Pond from '$lib/components/Pond.svelte'

    const connectionState = $derived(getConnectionState())
</script>

<svelte:head>
    <title>Pond</title>
    <meta name="description" content="Interactive water droplet sound installation" />
</svelte:head>

{#if connectionState === 'disconnected'}
    <RoomEntry />
{:else}
    <Pond />
{/if}
