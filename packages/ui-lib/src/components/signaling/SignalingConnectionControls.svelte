<script lang="ts">
	import classNames from 'classnames';
	import { getSignalingUrl } from 'ui-lib/lib/api-base';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import { signalingAdapter } from 'ui-lib/adapters/classes/signaling.adapter';
	import type { SignalingConnectionPhase } from 'ui-lib/types/signaling.type';

	const chatStore = signalingChatService.state;

	let {
		onConnect
	}: {
		onConnect?: () => void;
	} = $props();

	let serverUrl = $state(getSignalingUrl());
	let serverAvailable = $state(true);

	let aggregatePhase = $derived.by((): SignalingConnectionPhase => {
		const rooms = Object.values($chatStore.rooms);
		if (rooms.length === 0) return 'disconnected';
		if (rooms.some((r) => r.phase === 'connected')) return 'connected';
		if (rooms.some((r) => r.phase === 'connecting' || r.phase === 'authenticated'))
			return 'connecting';
		if (rooms.some((r) => r.phase === 'error')) return 'error';
		return 'disconnected';
	});

	function handleConnect() {
		if (!serverUrl) return;
		onConnect?.();
	}

	function handleDisconnect() {
		signalingChatService.disconnect();
	}
</script>

<div class="card flex flex-col bg-base-200">
	<div class="card-body gap-4 p-4">
		<div class="flex items-center justify-between">
			<h2 class="card-title text-base">Connection</h2>
			<span class={classNames('badge badge-sm', signalingAdapter.phaseBadgeClass(aggregatePhase))}>
				{signalingAdapter.phaseLabel(aggregatePhase)}
			</span>
		</div>

		<!-- Server URL display -->
		<div class="flex items-center gap-2">
			<span
				class={classNames('h-2 w-2 rounded-full', {
					'bg-success': serverAvailable,
					'bg-error': !serverAvailable
				})}
			></span>
			<span class="truncate font-mono text-xs text-base-content/60">
				{serverUrl || 'Not available'}
			</span>
		</div>

		<!-- Connect / Disconnect -->
		{#if aggregatePhase === 'disconnected' || aggregatePhase === 'error'}
			<button
				class="btn btn-sm btn-primary"
				disabled={!serverUrl || !serverAvailable || !onConnect}
				onclick={handleConnect}
			>
				Connect
			</button>
		{:else}
			<button class="btn btn-outline btn-sm btn-error" onclick={handleDisconnect}>
				Disconnect
			</button>
		{/if}

		<!-- Error display -->
		{#if $chatStore.error}
			<p class="text-sm text-error">{$chatStore.error}</p>
		{/if}

		<!-- Connected Peers -->
		{#if $chatStore.peerIds.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide text-base-content/50 uppercase">
					Peers ({$chatStore.peerIds.length})
				</h3>
				<div class="flex flex-wrap gap-1">
					{#each $chatStore.peerIds as peerId (peerId)}
						<span class="badge badge-outline font-mono badge-sm">
							{signalingAdapter.shortAddress(peerId)}
						</span>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Local Peer ID -->
		{#if $chatStore.localPeerId}
			<p class="text-xs text-base-content/40">
				You: <span class="font-mono">{signalingAdapter.shortAddress($chatStore.localPeerId)}</span>
			</p>
		{/if}
	</div>
</div>
