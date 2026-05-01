<script lang="ts">
	import { signalingChatService } from '$services/signaling-chat.service';
	import PeerSidebar from '$components/signaling/PeerSidebar.svelte';
	import SignalingChat from '$components/signaling/SignalingChat.svelte';
	import type { PeerConnectionStatus, SignalingPeerInfo } from '$types/signaling.type';

	const chatStore = signalingChatService.state;

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

<div class="flex h-full flex-col overflow-hidden rounded-xl border border-base-300 md:flex-row">
	<PeerSidebar
		roomPeers={allRoomPeers}
		peerConnectionStates={allPeerConnectionStates}
		activePeerId={$chatStore.activePeerId}
		localPeerId={$chatStore.localPeerId}
		onPeerClick={handlePeerClick}
		onPeerDisconnect={handlePeerDisconnect}
	/>
	<div class="flex flex-1 flex-col overflow-hidden">
		<SignalingChat />
	</div>
</div>
