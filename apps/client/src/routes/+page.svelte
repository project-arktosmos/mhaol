<script lang="ts">
	import { onDestroy } from 'svelte';
	import ConnectionDashboard from 'ui-lib/components/signaling/ConnectionDashboard.svelte';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import type { PeerConnectionStatus, SignalingPeerInfo } from 'ui-lib/types/signaling.type';

	const chatStore = signalingChatService.state;

	onDestroy(() => {
		signalingChatService.destroy();
	});

	// Aggregate roomPeers and peerConnectionStates across all rooms
	let allRoomPeers = $derived.by((): SignalingPeerInfo[] => {
		const seen = new Set<string>();
		const result: SignalingPeerInfo[] = [];
		for (const room of Object.values($chatStore.rooms)) {
			for (const peer of room.roomPeers) {
				if (!seen.has(peer.peer_id)) {
					seen.add(peer.peer_id);
					result.push(peer);
				}
			}
		}
		return result;
	});

	let allPeerConnectionStates = $derived.by((): Record<string, PeerConnectionStatus> => {
		const result: Record<string, PeerConnectionStatus> = {};
		for (const room of Object.values($chatStore.rooms)) {
			Object.assign(result, room.peerConnectionStates);
		}
		return result;
	});

	function handlePeerClick(peerId: string) {
		if (allPeerConnectionStates[peerId] === 'connected') {
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
		roomPeers={allRoomPeers}
		peerConnectionStates={allPeerConnectionStates}
		onPeerClick={handlePeerClick}
		onPeerDisconnect={handlePeerDisconnect}
	/>
</div>
