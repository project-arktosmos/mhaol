<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import { get } from 'svelte/store';
	import { connectionConfigService } from 'ui-lib/services/connection-config.service';
	import { nodeConnectionService } from 'ui-lib/services/node-connection.service';
	import SetupModalContent from './SetupModalContent.svelte';

	let {
		children
	}: {
		children?: Snippet;
	} = $props();

	const configStore = connectionConfigService.store;
	let configured = $derived($configStore !== null);

	let reconnecting = $state(false);
	let reconnectError = $state<string | null>(null);

	onMount(() => {
		const config = get(configStore);
		if (config) {
			reconnecting = true;
			const promise =
				config.transportMode === 'http'
					? nodeConnectionService.connectHttp(config)
					: nodeConnectionService.connectWebRtc(config);

			promise
				.then(() => {
					reconnecting = false;
				})
				.catch((err) => {
					reconnecting = false;
					reconnectError =
						err instanceof Error ? err.message : 'Failed to reconnect';
					// Clear config so the user sees the setup modal again
					connectionConfigService.clear();
				});
		}
	});

	function handleConnected() {
		// Config is saved by SetupModalContent — children will now render
	}
</script>

{#if reconnecting}
	<div class="flex h-full items-center justify-center">
		<div class="flex flex-col items-center gap-3">
			<span class="loading loading-lg loading-spinner text-primary"></span>
			<p class="text-sm text-base-content/60">Reconnecting to node...</p>
		</div>
	</div>
{:else if !configured}
	<div class="modal-open modal" role="dialog" aria-modal="true">
		<div class="modal-box max-h-[90vh] max-w-md overflow-y-auto">
			{#if reconnectError}
				<div class="alert alert-warning mb-4 text-sm">
					<span>Previous connection failed: {reconnectError}</span>
				</div>
			{/if}
			<SetupModalContent onconnected={handleConnected} />
		</div>
		<div class="modal-backdrop"></div>
	</div>
{:else}
	{@render children?.()}
{/if}
