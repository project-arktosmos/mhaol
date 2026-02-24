<script lang="ts">
	import classNames from 'classnames';
	import { createEventDispatcher } from 'svelte';
	import SignalingStatus from '$components/signaling/SignalingStatus.svelte';
	import type { SignalingServer, ServerStatus } from '$types/signaling.type';

	export let server: SignalingServer;
	export let status: ServerStatus | undefined = undefined;

	const dispatch = createEventDispatcher<{
		refresh: { server: SignalingServer };
		remove: { server: SignalingServer };
	}>();

	$: isDefault = server.id === 'local-default';
	$: online = status?.online ?? false;
	$: checking = status?.checking ?? false;
	$: wsConnected = status?.wsConnected ?? false;
	$: ownPeerId = status?.ownPeerId ?? null;
	$: lobbyPeers = status?.lobbyPeers ?? [];
	$: lastChecked = status?.lastChecked
		? new Date(status.lastChecked).toLocaleTimeString()
		: null;

	function shortId(id: string): string {
		// Ethereum address: 0x1234…abcd
		if (id.startsWith('0x') && id.length >= 10) {
			return `${id.slice(0, 6)}…${id.slice(-4)}`;
		}
		// Fallback for non-address IDs
		return id.slice(0, 8);
	}
</script>

<div class="card bg-base-200">
	<div class="card-body gap-3 p-4">
		<!-- Header row -->
		<div class="flex items-start justify-between gap-2">
			<div class="flex min-w-0 flex-col gap-1">
				<div class="flex items-center gap-2">
					<span class="font-semibold">{server.name}</span>
					{#if isDefault}
						<span class="badge badge-neutral badge-xs">local</span>
					{/if}
				</div>
				<span class="truncate text-xs text-base-content/50">{server.url}</span>
			</div>

			<div class="flex shrink-0 items-center gap-2">
				<SignalingStatus {online} {checking} />
				<button
					class="btn btn-ghost btn-xs"
					title="Refresh status"
					on:click={() => dispatch('refresh', { server })}
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-3.5 w-3.5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
						/>
					</svg>
				</button>
				{#if !isDefault}
					<button
						class="btn btn-ghost btn-xs text-error"
						title="Remove server"
						on:click={() => dispatch('remove', { server })}
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-3.5 w-3.5"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							<path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
						</svg>
					</button>
				{/if}
			</div>
		</div>

		<!-- Live peer presence section -->
		{#if wsConnected && ownPeerId}
			<div class="flex flex-col gap-2 rounded-lg bg-base-300 p-3">
				<div class="flex items-center gap-1.5 text-xs font-medium text-base-content/70">
					<div class="h-1.5 w-1.5 rounded-full bg-success"></div>
					Lobby ({lobbyPeers.length + 1} connected)
				</div>

				<!-- Own entry -->
				<div class="flex items-center gap-2 text-xs">
					<span class="badge badge-primary badge-xs">you</span>
					<span class="font-mono text-base-content/60">{shortId(ownPeerId)}…</span>
				</div>

				<!-- Other peers -->
				{#each lobbyPeers as peerId (peerId)}
					<div class="flex items-center gap-2 text-xs">
						<span class="badge badge-ghost badge-xs">peer</span>
						<span class="font-mono text-base-content/60">{shortId(peerId)}…</span>
					</div>
				{/each}

				{#if lobbyPeers.length === 0}
					<p class="text-xs text-base-content/40">No other browsers in this lobby yet</p>
				{/if}
			</div>
		{:else if online && !wsConnected}
			<p class="text-xs text-base-content/40">Connecting to lobby…</p>
		{/if}

		<!-- HTTP stats row (shown when connected) -->
		{#if online && status}
			<div class="flex flex-wrap gap-2">
				<span
					class={classNames('badge badge-sm', {
						'badge-ghost': status.totalPeers === 0,
						'badge-info': status.totalPeers > 0
					})}
				>
					{status.totalPeers}
					{status.totalPeers === 1 ? 'peer' : 'peers'} total
				</span>
				<span class="badge badge-ghost badge-sm">
					{status.rooms.length}
					{status.rooms.length === 1 ? 'room' : 'rooms'}
				</span>
			</div>
		{/if}

		{#if status?.error && !online}
			<p class="text-xs text-error/80">{status.error}</p>
		{/if}

		{#if lastChecked}
			<p class="text-xs text-base-content/40">Last checked: {lastChecked}</p>
		{/if}
	</div>
</div>
