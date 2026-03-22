<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import PeerChatLayout from 'ui-lib/components/signaling/PeerChatLayout.svelte';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import { rosterService } from 'ui-lib/services/roster.service';
	import { get } from 'svelte/store';

	onMount(() => {
		const { signalingServerUrl, signalingRoomId } = get(rosterService.state);
		signalingChatService.connect(signalingServerUrl, signalingRoomId);
	});

	onDestroy(() => {
		signalingChatService.destroy();
	});
</script>

<div class="flex h-[calc(100vh-5rem)] flex-col">
	<div class="mb-2">
		<h1 class="text-2xl font-bold">Peer Chat</h1>
		<p class="text-sm text-base-content/60">
			Select a peer to connect via WebRTC and chat directly
		</p>
	</div>
	<div class="min-h-0 flex-1">
		<PeerChatLayout />
	</div>
</div>
