<script lang="ts">
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import PeerSidebar from 'ui-lib/components/signaling/PeerSidebar.svelte';
	import SignalingChat from 'ui-lib/components/signaling/SignalingChat.svelte';

	const chatStore = signalingChatService.state;

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

<div class="flex h-full flex-col overflow-hidden rounded-xl border border-base-300 md:flex-row">
	<PeerSidebar
		roomPeerIds={$chatStore.roomPeerIds}
		peerConnectionStates={$chatStore.peerConnectionStates}
		activePeerId={$chatStore.activePeerId}
		localPeerId={$chatStore.localPeerId}
		onPeerClick={handlePeerClick}
		onPeerDisconnect={handlePeerDisconnect}
	/>
	<div class="flex flex-1 flex-col overflow-hidden">
		<SignalingChat />
	</div>
</div>
