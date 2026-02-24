<script lang="ts">
	import { walletService } from '$services/wallet.service';

	const state = walletService.state;

	let copied = false;

	async function copyAddress() {
		const address = $state.address;
		if (!address) return;
		await navigator.clipboard.writeText(address);
		copied = true;
		setTimeout(() => (copied = false), 1500);
	}

	function shortAddress(addr: string): string {
		if (!addr.startsWith('0x') || addr.length < 10) return addr;
		return `${addr.slice(0, 6)}…${addr.slice(-4)}`;
	}
</script>

<div class="flex items-center gap-2">
	{#if $state.loading}
		<span class="loading loading-spinner loading-xs"></span>
		<span class="text-sm text-base-content/50">Loading wallet…</span>
	{:else if $state.error}
		<span class="text-sm text-error">{$state.error}</span>
	{:else if $state.address}
		<div class="flex items-center gap-2 rounded-lg bg-base-200 px-3 py-1.5">
			<div class="h-2 w-2 rounded-full bg-success"></div>
			<span class="font-mono text-sm">{shortAddress($state.address)}</span>
			<button
				class="btn btn-ghost btn-xs h-auto min-h-0 px-1 py-0.5"
				title="Copy full address"
				on:click={copyAddress}
			>
				{#if copied}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-3.5 w-3.5 text-success"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
					</svg>
				{:else}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-3.5 w-3.5 opacity-50"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
						/>
					</svg>
				{/if}
			</button>
		</div>
	{/if}
</div>
