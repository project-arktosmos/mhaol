<script lang="ts">
	import classNames from 'classnames';
	import { signalingAdapter } from 'frontend/adapters/classes/signaling.adapter';
	import type { PeerConnectionStatus } from 'frontend/types/signaling.type';

	let {
		roomPeerIds = [],
		peerConnectionStates = {},
		activePeerId = null,
		localPeerId = null,
		onPeerClick,
		onPeerDisconnect
	}: {
		roomPeerIds: string[];
		peerConnectionStates: Record<string, PeerConnectionStatus>;
		activePeerId: string | null;
		localPeerId: string | null;
		onPeerClick: (peerId: string) => void;
		onPeerDisconnect: (peerId: string) => void;
	} = $props();

	function statusDotClass(peerId: string): string {
		const status = peerConnectionStates[peerId] ?? 'idle';
		const map: Record<PeerConnectionStatus, string> = {
			idle: 'bg-base-content/30',
			offering: 'bg-warning animate-pulse',
			answering: 'bg-warning animate-pulse',
			connected: 'bg-success',
			failed: 'bg-error'
		};
		return map[status];
	}
</script>

<div
	class="flex h-1/2 shrink-0 flex-col border-b border-base-300 bg-base-200 md:h-full md:w-64 md:border-r md:border-b-0"
>
	<div class="flex items-center justify-between border-b border-base-300 p-3">
		<h3 class="text-sm font-semibold">Peers</h3>
		{#if roomPeerIds.length > 0}
			<span class="badge badge-ghost badge-sm">{roomPeerIds.length}</span>
		{/if}
	</div>

	<div class="flex-1 overflow-y-auto">
		{#if roomPeerIds.length === 0}
			<p class="p-4 text-center text-xs text-base-content/40">No peers online</p>
		{:else}
			<div class="flex flex-col">
				{#each roomPeerIds as peerId (peerId)}
					{@const status = peerConnectionStates[peerId] ?? 'idle'}
					{@const isActive = peerId === activePeerId}
					<div
						class={classNames(
							'flex cursor-pointer items-center gap-2 px-3 py-2 text-left transition-colors',
							{
								'bg-primary/10': isActive,
								'hover:bg-base-300': !isActive
							}
						)}
						role="button"
						tabindex="0"
						onclick={() => onPeerClick(peerId)}
						onkeydown={(e: KeyboardEvent) => {
							if (e.key === 'Enter' || e.key === ' ') onPeerClick(peerId);
						}}
					>
						<span class={classNames('h-2.5 w-2.5 shrink-0 rounded-full', statusDotClass(peerId))}
						></span>
						<div class="min-w-0 flex-1">
							<span class="block truncate font-mono text-xs">
								{signalingAdapter.shortAddress(peerId)}
							</span>
							<span class="text-[10px] text-base-content/40">
								{signalingAdapter.peerConnectionStatusLabel(status)}
							</span>
						</div>
						{#if status === 'connected'}
							<button
								class="btn text-error btn-ghost btn-xs"
								onclick={(e: MouseEvent) => {
									e.stopPropagation();
									onPeerDisconnect(peerId);
								}}
								title="Disconnect"
							>
								x
							</button>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>

	{#if localPeerId}
		<div class="border-t border-base-300 p-3">
			<p class="text-[10px] text-base-content/40">
				You: <span class="font-mono">{signalingAdapter.shortAddress(localPeerId)}</span>
			</p>
		</div>
	{/if}
</div>
