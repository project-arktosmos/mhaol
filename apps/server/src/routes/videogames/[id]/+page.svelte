<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { gameExtendedToDisplay } from 'addons/retroachievements';
	import type { RaGameMetadata, RaGameExtended } from 'addons/retroachievements/types';
	import GameDetailPage from 'ui-lib/components/videogames/GameDetailPage.svelte';

	let game = $state<RaGameMetadata | null>(null);
	let details = $state<RaGameMetadata | null>(null);
	let detailsLoading = $state(true);

	let id = $derived($page.params.id ?? '');

	async function fetchGame(gameId: string) {
		detailsLoading = true;
		try {
			const res = await fetch(apiUrl(`/api/retroachievements/games/${gameId}`));
			if (!res.ok) throw new Error('Failed to fetch game');
			const data = await res.json();
			const detail = gameExtendedToDisplay(data as RaGameExtended);
			game = detail;
			details = detail;
		} catch {
			game = null;
			details = null;
		}
		detailsLoading = false;
	}

	onMount(() => {
		fetchGame(id);
	});
</script>

{#if game}
	<GameDetailPage {game} {details} {detailsLoading} onback={() => goto('/videogames')} />
{:else if detailsLoading}
	<div class="flex flex-1 items-center justify-center">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Game not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto('/videogames')}>Back to games</button>
	</div>
{/if}
