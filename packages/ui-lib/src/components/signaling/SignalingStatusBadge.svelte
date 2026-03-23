<script lang="ts">
	import classNames from 'classnames';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
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

	let peerCount = $derived(
		new Set(Object.values($chatStore.rooms).flatMap((r) => r.roomPeers.map((p) => p.peer_id)))
			.size
	);

	let roomCount = $derived(
		Object.values($chatStore.rooms).filter((r) => r.phase === 'connected').length
	);
</script>

<div class="flex items-center gap-1.5" title={signalingAdapter.phaseLabel(aggregatePhase)}>
	<span
		class={classNames('h-2 w-2 rounded-full', {
			'bg-success': aggregatePhase === 'connected',
			'animate-pulse bg-info':
				aggregatePhase === 'connecting' || aggregatePhase === 'authenticated',
			'bg-error': aggregatePhase === 'error',
			'bg-base-content/30': aggregatePhase === 'disconnected'
		})}
	></span>
	<span class="hidden text-xs text-base-content/60 sm:inline">
		{signalingAdapter.phaseLabel(aggregatePhase)}
		{#if roomCount > 1}({roomCount} rooms){/if}
	</span>
	{#if aggregatePhase === 'connected' && peerCount > 0}
		<span class="badge badge-xs badge-success">{peerCount}</span>
	{/if}
</div>
