<script lang="ts">
	import classNames from 'classnames';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import { DEFAULT_SIGNALING_URL } from 'ui-lib/lib/api-base';
	import { signalingAdapter } from 'ui-lib/adapters/classes/signaling.adapter';
	import type { SignalingConnectionPhase } from 'ui-lib/types/signaling.type';

	const chatStore = signalingChatService.state;

	let aggregatePhase = $derived.by((): SignalingConnectionPhase => {
		const rooms = Object.values($chatStore.rooms);
		if (rooms.length === 0) return 'disconnected';
		if (rooms.some((r) => r.phase === 'connected')) return 'connected';
		if (rooms.some((r) => r.phase === 'connecting' || r.phase === 'authenticated'))
			return 'connecting';
		if (rooms.some((r) => r.phase === 'error')) return 'error';
		return 'disconnected';
	});

	let roomNames = $derived(Object.keys($chatStore.rooms));
</script>

<div class="pr-8">
	<h3 class="text-lg font-bold">Signaling</h3>
	<p class="text-sm text-base-content/60">WebRTC signaling connection info</p>
</div>

<div class="mt-4 flex flex-col gap-4">
	<div class="card bg-base-200">
		<div class="card-body gap-3 p-4">
			<div class="flex items-center justify-between">
				<h2 class="card-title text-base">Connection</h2>
				<span
					class={classNames('badge badge-sm', signalingAdapter.phaseBadgeClass(aggregatePhase))}
				>
					{signalingAdapter.phaseLabel(aggregatePhase)}
				</span>
			</div>

			<div class="flex flex-col gap-2">
				<div>
					<span class="text-xs font-semibold tracking-wide text-base-content/50 uppercase"
						>Server URL</span
					>
					<div class="mt-1 flex items-center gap-2">
						<span
							class={classNames('h-2 w-2 rounded-full', {
								'bg-success': aggregatePhase === 'connected',
								'bg-warning': aggregatePhase === 'connecting' || aggregatePhase === 'authenticated',
								'bg-error': aggregatePhase === 'disconnected' || aggregatePhase === 'error'
							})}
						></span>
						<span class="truncate font-mono text-xs text-base-content/60">
							{DEFAULT_SIGNALING_URL}
						</span>
					</div>
				</div>

				<div>
					<span class="text-xs font-semibold tracking-wide text-base-content/50 uppercase"
						>Rooms</span
					>
					<div class="mt-1 flex flex-wrap gap-1">
						{#each roomNames as roomName (roomName)}
							{@const room = $chatStore.rooms[roomName]}
							<span
								class={classNames(
									'badge font-mono badge-sm',
									signalingAdapter.phaseBadgeClass(room.phase)
								)}
							>
								{roomName.length > 12 ? signalingAdapter.shortAddress(roomName) : roomName}
							</span>
						{/each}
						{#if roomNames.length === 0}
							<span class="font-mono text-xs text-base-content/60">No rooms</span>
						{/if}
					</div>
				</div>

				{#if $chatStore.localPeerId}
					<div>
						<span class="text-xs font-semibold tracking-wide text-base-content/50 uppercase"
							>Your Address</span
						>
						<p class="mt-1 font-mono text-sm">{$chatStore.localPeerId}</p>
					</div>
				{/if}
			</div>
		</div>
	</div>

	{#if $chatStore.peerIds.length > 0}
		<div class="card bg-base-200">
			<div class="card-body gap-2 p-4">
				<h2 class="card-title text-base">Peers ({$chatStore.peerIds.length})</h2>
				<div class="flex flex-wrap gap-1">
					{#each $chatStore.peerIds as peerId (peerId)}
						<span class="badge badge-outline font-mono badge-sm">
							{signalingAdapter.shortAddress(peerId)}
						</span>
					{/each}
				</div>
			</div>
		</div>
	{/if}

	{#if $chatStore.error}
		<div class="alert alert-error">
			<span>{$chatStore.error}</span>
		</div>
	{/if}
</div>
