<script lang="ts">
	import { onDestroy } from 'svelte';
	import SignalingConnectionControls from 'ui-lib/components/signaling/SignalingConnectionControls.svelte';
	import SignalingChat from 'ui-lib/components/signaling/SignalingChat.svelte';
	import { signalingChatService } from 'frontend/services/signaling-chat.service';

	onDestroy(() => {
		signalingChatService.destroy();
	});
</script>

<div class="mx-auto flex max-w-4xl flex-col gap-6">
	<div>
		<h1 class="text-2xl font-bold">Peer Chat</h1>
		<p class="text-sm text-base-content/60">
			Connect to a signaling server and chat with peers over WebRTC
		</p>
	</div>

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
		<SignalingConnectionControls />
		<SignalingChat />
	</div>
</div>
