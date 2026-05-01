<script lang="ts">
	import classNames from 'classnames';
	import { nodeConnectionService } from '$services/node-connection.service';
	import { connectionConfigService } from '$services/connection-config.service';
	import { clientIdentityService } from '$services/client-identity.service';
	import { signalingAdapter } from '$adapters/classes/signaling.adapter';

	let { onclick }: { onclick?: () => void } = $props();

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

<button class="flex cursor-pointer items-center gap-1.5" title={statusLabel} {onclick}>
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
</button>
