<script lang="ts">
	import { onDestroy } from 'svelte';
	import { signalingChatService } from '$services/signaling-chat.service';
	import SignalingServerStatus from '$components/signaling/SignalingServerStatus.svelte';
	import SignalingConnectionControls from '$components/signaling/SignalingConnectionControls.svelte';
	import SignalingChat from '$components/signaling/SignalingChat.svelte';

	onDestroy(() => {
		signalingChatService.destroy();
	});
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Signaling Server</h3>
	<p class="text-sm text-base-content/60">
		Manage the WebRTC signaling server and test peer-to-peer chat
	</p>
</div>

<div class="mt-4 flex flex-col gap-3">
	<h4 class="text-base font-semibold">Server Management</h4>
	<SignalingServerStatus />
</div>

<div class="mt-6 flex flex-col gap-3">
	<h4 class="text-base font-semibold">WebRTC Test Chat</h4>
	<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
		<SignalingConnectionControls />
		<SignalingChat />
	</div>
</div>
