<script lang="ts">
	import { onMount } from 'svelte';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { p2pStreamService } from 'ui-lib/services/p2p-stream.service';
	import P2pStreamSettings from 'ui-lib/components/p2p-stream/P2pStreamSettings.svelte';

	let resetting = $state(false);
	let error = $state<string | null>(null);

	onMount(() => {
		p2pStreamService.initialize();
	});

	async function handleReset() {
		resetting = true;
		error = null;

		try {
			const res = await fetchRaw('/api/database/reset', { method: 'POST' });
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

<div class="pr-8">
	<h3 class="text-lg font-bold">Settings</h3>
	<p class="text-sm text-base-content/60">Application configuration and maintenance</p>
</div>

{#if error}
	<div class="mt-4 alert alert-error">
		<span>{error}</span>
		<button class="btn btn-ghost btn-sm" onclick={() => (error = null)}>x</button>
	</div>
{/if}

<div class="mt-4">
	<P2pStreamSettings />
</div>

<div class="card mt-4 bg-base-200">
	<div class="card-body">
		<h2 class="card-title text-lg text-error">Danger Zone</h2>

		<div class="mt-2 flex items-center justify-between rounded-lg border border-error/30 p-4">
			<div>
				<h3 class="font-semibold">Reset Database</h3>
				<p class="text-sm opacity-70">
					Drop all tables, recreate from schema, and reseed defaults.
				</p>
			</div>
			<button class="btn btn-sm btn-error" disabled={resetting} onclick={handleReset}>
				{#if resetting}
					<span class="loading loading-sm loading-spinner"></span>
				{:else}
					Reset Database
				{/if}
			</button>
		</div>
	</div>
</div>
