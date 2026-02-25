<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { walletService } from '$services/wallet.service';
	import { p2pService } from '$services/p2p.service';
	import WalletDisplay from '$components/p2p/WalletDisplay.svelte';
	import P2pSdpExchange from '$components/p2p/P2pSdpExchange.svelte';
	import P2pChat from '$components/p2p/P2pChat.svelte';

	const state = p2pService.state;

	onMount(async () => {
		await walletService.initialize();
		await p2pService.initialize();
	});

	onDestroy(() => {
		p2pService.destroy();
	});
</script>

<div class="flex flex-col gap-6 p-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">P2P Chat</h1>
			<p class="text-sm text-base-content/60">Serverless peer-to-peer messaging via WebRTC</p>
		</div>
		{#if !$state.initialized}
			<span class="loading loading-spinner loading-md"></span>
		{/if}
	</div>

	<!-- Wallet identity -->
	<div class="flex items-center gap-3">
		<span class="text-sm text-base-content/50">Your identity</span>
		<WalletDisplay />
	</div>

	<!-- Main layout -->
	<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
		<P2pSdpExchange />
		<P2pChat />
	</div>
</div>
