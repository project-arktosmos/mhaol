<script lang="ts">
	import { peerLibraryService } from 'ui-lib/services/peer-library.service';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import PeerLibraryList from './PeerLibraryList.svelte';

	const peerState = peerLibraryService.state;
	const signalingState = signalingChatService.state;

	let isConnected = $derived(
		Object.values($signalingState.rooms).some((r) => r.phase === 'connected')
	);
	let peers = $derived(Object.entries($peerState.peers));
	let peerCount = $derived(peers.length);
</script>

<div class="flex flex-col gap-4">
	<div class="flex items-center justify-between">
		<h3 class="text-lg font-bold">Peer Libraries</h3>
		{#if isConnected && peerCount > 0}
			<span class="badge badge-sm badge-info">{peerCount} peer{peerCount !== 1 ? 's' : ''}</span>
		{/if}
	</div>

	{#if !isConnected}
		<div class="rounded-lg bg-base-200 p-6 text-center">
			<p class="text-sm opacity-60">Not connected to a signaling room.</p>
			<p class="mt-1 text-xs opacity-40">Connect via the Signaling modal to see peer libraries.</p>
		</div>
	{:else if peerCount === 0}
		<div class="rounded-lg bg-base-200 p-6 text-center">
			<p class="text-sm opacity-60">No peers connected yet.</p>
			<p class="mt-1 text-xs opacity-40">Waiting for other peers to join the room...</p>
		</div>
	{:else}
		<div class="flex flex-col gap-3">
			{#each peers as [peerId, peerData] (peerId)}
				<PeerLibraryList {peerId} {peerData} />
			{/each}
		</div>
	{/if}
</div>
