<script lang="ts">
	import { onMount, tick, type Snippet } from 'svelte';
	import { get } from 'svelte/store';
	import { connectionConfigService } from 'ui-lib/services/connection-config.service';
	import { nodeConnectionService } from 'ui-lib/services/node-connection.service';
	import { extractInviteFromUrl, clearInviteFromUrl } from 'ui-lib/services/connect-invite.service';
	import SetupModalContent from './SetupModalContent.svelte';

	let {
		children,
		onready
	}: {
		children?: Snippet;
		onready?: () => void | Promise<void>;
	} = $props();

	const configStore = connectionConfigService.store;
	let configured = $derived($configStore !== null);

	let reconnecting = $state(false);
	let reconnectError = $state<string | null>(null);
	let urlInvite = $state<string | null>(null);

	function connectWith(config: import('ui-lib/types/connection-config.type').ConnectionConfig) {
		reconnecting = true;
		let promise: Promise<void>;

		if (config.transportMode === 'ws') {
			promise = nodeConnectionService.connectWs(config);
		} else if (config.transportMode === 'webrtc') {
			promise = nodeConnectionService.connectWebRtc(config);
		} else {
			promise = nodeConnectionService.connectHttp(config);
		}

		promise
			.then(async () => {
				reconnecting = false;
				connectionConfigService.save(config);
				await tick();
				await onready?.();
			})
			.catch((err) => {
				reconnecting = false;
				reconnectError = err instanceof Error ? err.message : 'Failed to reconnect';
				connectionConfigService.clear();
			});
	}

	onMount(() => {
		urlInvite = extractInviteFromUrl();
		clearInviteFromUrl();

		const config = get(configStore);
		if (config) {
			connectWith(config);
		}
	});

	async function handleConnected() {
		// Config is saved by SetupModalContent — wait for children to render, then re-fetch with auth
		await tick();
		await onready?.();
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
				<div class="mb-4 alert text-sm alert-warning">
					<span>Previous connection failed: {reconnectError}</span>
				</div>
			{/if}
			<SetupModalContent onconnected={handleConnected} initialInvite={urlInvite ?? undefined} />
		</div>
		<div class="modal-backdrop"></div>
	</div>
{:else}
	{@render children?.()}
{/if}
