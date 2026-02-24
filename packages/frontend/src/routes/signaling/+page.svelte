<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { signalingService } from '$services/signaling.service';
	import { walletService } from '$services/wallet.service';
	import type { SignalingServer } from '$types/signaling.type';
	import SignalingServerList from '$components/signaling/SignalingServerList.svelte';
	import SignalingAddServer from '$components/signaling/SignalingAddServer.svelte';
	import WalletDisplay from '$components/signaling/WalletDisplay.svelte';

	const state = signalingService.state;
	const servers = signalingService.store;

	onMount(async () => {
		// Wallet must be ready before signaling connects (needs it to sign challenges)
		await walletService.initialize();
		await signalingService.initialize();
	});

	onDestroy(() => {
		signalingService.destroy();
	});

	function handleAdd(event: CustomEvent<{ name: string; url: string }>) {
		signalingService.addServer(event.detail.name, event.detail.url);
	}

	function handleRefresh(event: CustomEvent<{ server: SignalingServer }>) {
		signalingService.checkServerStatus(event.detail.server);
	}

	function handleRemove(event: CustomEvent<{ server: SignalingServer }>) {
		signalingService.removeServer(event.detail.server);
	}
</script>

<div class="flex flex-col gap-6 p-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-2xl font-bold">Signaling</h1>
			<p class="text-sm text-base-content/60">Manage WebRTC signaling servers</p>
		</div>
		<div class="flex items-center gap-4">
			{#if !$state.initialized}
				<span class="loading loading-spinner loading-md"></span>
			{/if}
			{#if !$state.showAddForm}
				<button class="btn btn-primary btn-sm" on:click={() => signalingService.openAddForm()}>
					Add Server
				</button>
			{/if}
		</div>
	</div>

	<!-- Wallet identity -->
	<div class="flex items-center gap-3">
		<span class="text-sm text-base-content/50">Your wallet</span>
		<WalletDisplay />
	</div>

	{#if $state.showAddForm}
		<SignalingAddServer on:add={handleAdd} on:cancel={() => signalingService.closeAddForm()} />
	{/if}

	<SignalingServerList
		servers={$servers}
		statuses={$state.serverStatuses}
		on:refresh={handleRefresh}
		on:remove={handleRemove}
	/>
</div>
