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

<div class="flex flex-col gap-6 p-6">
	<!-- Header -->
	<div>
		<h1 class="text-2xl font-bold">Signaling Server</h1>
		<p class="text-sm text-base-content/60">
			Manage the WebRTC signaling server and test peer-to-peer chat
		</p>
	</div>

	<!-- Server Management -->
	<div class="flex flex-col gap-3">
		<h2 class="text-lg font-semibold">Server Management</h2>
		<SignalingServerStatus />
	</div>

	<!-- WebRTC Test Chat -->
	<div class="flex flex-col gap-3">
		<h2 class="text-lg font-semibold">WebRTC Test Chat</h2>
		<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
			<SignalingConnectionControls />
			<SignalingChat />
		</div>
	</div>
</div>
