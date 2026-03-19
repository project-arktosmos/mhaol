<script lang="ts">
	import { onDestroy } from 'svelte';
	import { signalingChatService } from 'frontend/services/signaling-chat.service';
	import SignalingServerStatus from 'ui-lib/components/signaling/SignalingServerStatus.svelte';
	import SignalingConnectionControls from 'ui-lib/components/signaling/SignalingConnectionControls.svelte';
	import SignalingChat from 'ui-lib/components/signaling/SignalingChat.svelte';

	onDestroy(() => {
		signalingChatService.destroy();
	});
</script>

<div class="flex flex-col gap-6 p-6">
	<div>
		<h1 class="text-2xl font-bold">Signaling Dashboard</h1>
		<p class="text-sm text-base-content/60">
			Manage signaling servers and test WebRTC peer-to-peer connections
		</p>
	</div>

	<section>
		<h2 class="mb-3 text-lg font-semibold">Server Status</h2>
		<SignalingServerStatus />
	</section>

	<section>
		<h2 class="mb-3 text-lg font-semibold">Connection Test</h2>
		<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
			<SignalingConnectionControls />
			<SignalingChat />
		</div>
	</section>
</div>
