<script lang="ts">
	import classNames from 'classnames';
	import { signalingChatService } from 'ui-lib/services/signaling-chat.service';
	import { signalingAdapter } from 'ui-lib/adapters/classes/signaling.adapter';

	const chatStore = signalingChatService.state;

	let peerCount = $derived($chatStore.roomPeers.length);
</script>

<div class="flex items-center gap-1.5" title={signalingAdapter.phaseLabel($chatStore.phase)}>
	<span
		class={classNames('h-2 w-2 rounded-full', {
			'bg-success': $chatStore.phase === 'connected',
			'animate-pulse bg-info':
				$chatStore.phase === 'connecting' || $chatStore.phase === 'authenticated',
			'bg-error': $chatStore.phase === 'error',
			'bg-base-content/30': $chatStore.phase === 'disconnected'
		})}
	></span>
	<span class="hidden text-xs text-base-content/60 sm:inline">
		{signalingAdapter.phaseLabel($chatStore.phase)}
	</span>
	{#if $chatStore.phase === 'connected' && peerCount > 0}
		<span class="badge badge-xs badge-success">{peerCount}</span>
	{/if}
</div>
