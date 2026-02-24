<script lang="ts">
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
	$: lastChecked = status?.lastChecked
		? new Date(status.lastChecked).toLocaleTimeString()
		: null;
</script>

<div class="card bg-base-200">
	<div class="card-body gap-3 p-4">
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
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M6 18L18 6M6 6l12 12"
							/>
						</svg>
					</button>
				{/if}
			</div>
		</div>

		{#if online && status}
			<div class="flex flex-wrap gap-2">
				<span class="badge badge-ghost badge-sm">
					{status.totalPeers}
					{status.totalPeers === 1 ? 'peer' : 'peers'}
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
