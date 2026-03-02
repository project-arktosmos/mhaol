<script lang="ts">
	import { onMount } from 'svelte';
	import { apiUrl } from '$lib/api-base';
	import { p2pStreamService } from '$services/p2p-stream.service';
	import P2pStreamSettings from '$components/p2p-stream/P2pStreamSettings.svelte';

	let resetting = $state(false);
	let error = $state<string | null>(null);

	onMount(async () => {
		await p2pStreamService.initialize();
	});

	async function handleReset() {
		resetting = true;
		error = null;

		try {
			const res = await fetch(apiUrl('/api/database/reset'), { method: 'POST' });
			if (!res.ok) {
				const body = await res.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`);
			}
			window.location.reload();
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			resetting = false;
		}
	}
</script>

<div class="container mx-auto p-4">
	<div class="mb-6">
		<h1 class="text-3xl font-bold">Settings</h1>
		<p class="text-sm opacity-70">Application configuration and maintenance</p>
	</div>

	{#if error}
		<div class="alert alert-error mb-4">
			<span>{error}</span>
			<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
		</div>
	{/if}

	<!-- P2P Streaming Settings -->
	<div class="mb-4">
		<P2pStreamSettings />
	</div>

	<div class="card bg-base-200">
		<div class="card-body">
			<h2 class="card-title text-lg text-error">Danger Zone</h2>

			<div class="mt-2 flex items-center justify-between rounded-lg border border-error/30 p-4">
				<div>
					<h3 class="font-semibold">Reset Database</h3>
					<p class="text-sm opacity-70">
						Drop all tables, recreate from schema, and reseed defaults.
					</p>
				</div>
				<button class="btn btn-error btn-sm" disabled={resetting} onclick={handleReset}>
					{#if resetting}
						<span class="loading loading-spinner loading-sm"></span>
					{:else}
						Reset Database
					{/if}
				</button>
			</div>
		</div>
	</div>
</div>
