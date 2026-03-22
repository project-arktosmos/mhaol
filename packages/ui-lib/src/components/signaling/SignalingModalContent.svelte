<script lang="ts">
	import { onDestroy } from 'svelte';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import SignalingConnectionControls from 'ui-lib/components/signaling/SignalingConnectionControls.svelte';
	import SignalingChat from 'ui-lib/components/signaling/SignalingChat.svelte';

	onDestroy(() => {
		signalingChatService.destroy();
	});
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Signaling Server</h3>
	<p class="text-sm text-base-content/60">WebRTC signaling server for peer-to-peer chat</p>
</div>

<div class="mt-4 flex flex-col gap-3">
	<SignalingConnectionControls />
	<SignalingChat />
</div>
