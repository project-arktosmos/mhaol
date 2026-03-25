<script lang="ts">
	import classNames from 'classnames';
	import { nodeConnectionService } from 'ui-lib/services/node-connection.service';
	import { connectionConfigService } from 'ui-lib/services/connection-config.service';
	import { clientIdentityService } from 'ui-lib/services/client-identity.service';
	import { signalingAdapter } from 'ui-lib/adapters/classes/signaling.adapter';

	const connState = nodeConnectionService.state;
	const configStore = connectionConfigService.store;

	let connected = $derived($connState.phase === 'ready');
	let connecting = $derived(
		$connState.phase !== 'idle' && $connState.phase !== 'ready' && $connState.phase !== 'error'
	);

	let transportLabel = $derived($configStore?.transportMode?.toUpperCase() ?? '');

	let statusLabel = $derived.by(() => {
		if (connected && transportLabel) return `Connected (${transportLabel})`;
		if (connected) return 'Connected';
		if (connecting) return 'Connecting...';
		return 'Disconnected';
	});

	const localIdentity = clientIdentityService.loadLocal();
	let shortAddress = localIdentity.address
		? signalingAdapter.shortAddress(localIdentity.address)
		: '';
</script>

<div class="dropdown dropdown-end dropdown-hover">
	<div
		tabindex="0"
		role="button"
		class="flex cursor-pointer items-center gap-1.5"
		title={statusLabel}
	>
		<span
			class={classNames('h-2 w-2 rounded-full', {
				'bg-success': connected,
				'animate-pulse bg-info': connecting,
				'bg-base-content/30': !connected && !connecting
			})}
		></span>
		<span class="hidden text-xs text-base-content/60 sm:inline">
			{statusLabel}
		</span>
	</div>
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<div
		tabindex="0"
		class="dropdown-content z-50 mt-2 w-64 rounded-box bg-base-200 p-3 shadow-lg"
	>
		<div class="flex flex-col gap-2 text-sm">
			<div>
				<span class="text-base-content/60">Name</span>
				<p class="mt-0.5 font-medium">{localIdentity.name}</p>
			</div>
			<div>
				<span class="text-base-content/60">Address</span>
				<p class="mt-0.5 truncate font-mono text-xs" title={localIdentity.address}>
					{shortAddress}
				</p>
			</div>
			{#if $configStore}
				<div>
					<span class="text-base-content/60">Transport</span>
					<p class="mt-0.5">{$configStore.transportMode.toUpperCase()}</p>
				</div>
			{/if}
		</div>
	</div>
</div>
