<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import SignalingServerItem from '$components/signaling/SignalingServerItem.svelte';
	import type { SignalingServer, ServerStatus } from '$types/signaling.type';

	export let servers: SignalingServer[] = [];
	export let statuses: Record<string, ServerStatus> = {};

	const dispatch = createEventDispatcher<{
		refresh: { server: SignalingServer };
		remove: { server: SignalingServer };
	}>();
</script>

{#if servers.length === 0}
	<div class="flex flex-col items-center gap-2 py-12 text-base-content/40">
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class="h-10 w-10"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="1.5"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				d="M5 12h14M12 5l7 7-7 7"
			/>
		</svg>
		<p class="text-sm">No signaling servers configured</p>
	</div>
{:else}
	<div class="flex flex-col gap-3">
		{#each servers as server (server.id)}
			<SignalingServerItem
				{server}
				status={statuses[String(server.id)]}
				on:refresh={(e) => dispatch('refresh', e.detail)}
				on:remove={(e) => dispatch('remove', e.detail)}
			/>
		{/each}
	</div>
{/if}
