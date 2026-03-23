<script lang="ts">
	import { onDestroy } from 'svelte';
	import ConnectionDashboard from 'ui-lib/components/signaling/ConnectionDashboard.svelte';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';

	const chatStore = signalingChatService.state;

	onDestroy(() => {
		signalingChatService.destroy();
	});

	function handlePeerClick(peerId: string) {
		const states = $chatStore.peerConnectionStates;
		if (states[peerId] === 'connected') {
			signalingChatService.setActivePeer(peerId);
		} else {
			signalingChatService.connectToPeer(peerId);
		}
	}

	function handlePeerDisconnect(peerId: string) {
		signalingChatService.disconnectPeer(peerId);
	}
</script>

<div class="mx-auto max-w-5xl">
	<div class="mb-4">
		<h1 class="text-2xl font-bold">Connection</h1>
		<p class="text-sm text-base-content/60">Connect to a server peer to browse its media catalog</p>
	</div>
	<ConnectionDashboard
		roomPeers={$chatStore.roomPeers}
		peerConnectionStates={$chatStore.peerConnectionStates}
		onPeerClick={handlePeerClick}
		onPeerDisconnect={handlePeerDisconnect}
	/>
</div>
