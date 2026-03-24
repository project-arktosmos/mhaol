<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { apiUrl } from 'ui-lib/lib/api-base';
	import { gameListItemToDisplay, gameExtendedToDisplay } from 'addons/retroachievements';
	import type { RaGameMetadata, RaGameListItem, RaGameExtended } from 'addons/retroachievements/types';
	import { RA_CONSOLES } from 'addons/retroachievements/types';
	import { CONSOLE_IMAGES } from 'assets/game-consoles';
	import GameCard from 'ui-lib/components/videogames/GameCard.svelte';
	import BrowseHeader from 'ui-lib/components/browse/BrowseHeader.svelte';
	import BrowseGrid from 'ui-lib/components/browse/BrowseGrid.svelte';
	import classNames from 'classnames';

	const PAGE_SIZE = 20;

	let selectedConsoleId = $state(5);
	let games = $state<RaGameMetadata[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let page = $state(0);
	let searchQuery = $state('');
	let selectedCategory = $state('Originals');

	function gameTag(title: string): string {
		if (title.startsWith('~')) {
			const end = title.indexOf('~', 1);
			if (end > 1) return title.substring(1, end);
		}
		return 'Originals';
	}

	let categoryTabs = $derived.by(() => {
		const counts = new Map<string, number>();
		for (const g of games) {
			const tag = gameTag(g.title);
			counts.set(tag, (counts.get(tag) ?? 0) + 1);
		}
		const tags: string[] = [];
		if (counts.has('Originals')) tags.push('Originals');
		for (const tag of [...counts.keys()].sort()) {
			if (tag !== 'Originals') tags.push(tag);
		}
		return tags;
	});

	let filteredGames = $derived.by(() => {
		let result = games.filter((g) => gameTag(g.title) === selectedCategory);
		if (searchQuery.trim()) {
			const lower = searchQuery.toLowerCase();
			result = result.filter((g) => g.title.toLowerCase().includes(lower));
		}
		return result;
	});

	let totalPages = $derived(Math.ceil(filteredGames.length / PAGE_SIZE));
	let pagedGames = $derived(filteredGames.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE));

	let consoleCache: Record<number, RaGameMetadata[]> = {};
	let detailsCache: Record<number, RaGameMetadata> = {};
	let backfillGeneration = 0;

	async function fetchGameList(consoleId: number) {
		if (consoleCache[consoleId]) { games = consoleCache[consoleId]; return; }
		loading = true;
		error = null;
		try {
			const res = await fetch(apiUrl(`/api/retroachievements/games?console=${consoleId}`));
			if (!res.ok) throw new Error('Failed to fetch game list');
			const data = await res.json();
			if (!Array.isArray(data)) throw new Error('Unexpected response format');
			const display = data.map((g: unknown) =>
				gameListItemToDisplay(g as RaGameListItem)
			);
			consoleCache[consoleId] = display;
			games = display;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
			games = [];
		}
		loading = false;
	}

	async function backfillPageImages() {
		const gen = ++backfillGeneration;
		const BATCH_SIZE = 5;
		const toFetch = pagedGames.filter((g) => !g.imageBoxArtUrl);
		for (let i = 0; i < toFetch.length; i += BATCH_SIZE) {
			if (backfillGeneration !== gen) return;
			const batch = toFetch.slice(i, i + BATCH_SIZE);
			await Promise.all(
				batch.map(async (game) => {
					if (detailsCache[game.id]) {
						game.imageBoxArtUrl = detailsCache[game.id].imageBoxArtUrl;
						game.imageTitleUrl = detailsCache[game.id].imageTitleUrl;
						game.imageIngameUrl = detailsCache[game.id].imageIngameUrl;
						return;
					}
					try {
						const res = await fetch(apiUrl(`/api/retroachievements/games/${game.id}`));
						if (!res.ok) return;
						const data = await res.json();
						const detail = gameExtendedToDisplay(
							data as RaGameExtended
						);
						detailsCache[game.id] = detail;
						game.imageBoxArtUrl = detail.imageBoxArtUrl;
						game.imageTitleUrl = detail.imageTitleUrl;
						game.imageIngameUrl = detail.imageIngameUrl;
					} catch { /* skip */ }
				})
			);
			if (backfillGeneration !== gen) return;
			games = [...games];
		}
	}

	$effect(() => {
		if (pagedGames.length > 0 && !loading) backfillPageImages();
	});

	function handleConsoleChange(consoleId: number) {
		selectedConsoleId = consoleId;
		searchQuery = '';
		selectedCategory = 'Originals';
		page = 0;
		fetchGameList(consoleId);
	}

	function handleSelectGame(game: RaGameMetadata) {
		goto(`/videogames/${game.id}`);
	}

	function handlePageChange(newPage: number) {
		page = newPage;
	}

	$effect(() => { searchQuery; page = 0; });

	onMount(() => { fetchGameList(selectedConsoleId); });
</script>

<div class="flex min-w-0 flex-1 flex-col overflow-hidden">
	<BrowseHeader title="Videogames" count={filteredGames.length} countLabel="games">
		{#snippet tabs()}
			{#each RA_CONSOLES as console}
				<button
					class={classNames('btn btn-xs', {
						'btn-primary': selectedConsoleId === console.id,
						'btn-ghost': selectedConsoleId !== console.id
					})}
					onclick={() => handleConsoleChange(console.id)}
				>
					{#if CONSOLE_IMAGES[console.id]}
						<img src={CONSOLE_IMAGES[console.id]} alt="" class="h-4 w-4" />
					{/if}
					{console.name}
				</button>
			{/each}
		{/snippet}
	</BrowseHeader>

	<!-- Category tabs + Search -->
	<div class="flex items-center gap-3 border-b border-base-300 px-4 py-2">
		{#each categoryTabs as tab}
			<button
				class={classNames('btn btn-xs', {
					'btn-secondary': selectedCategory === tab,
					'btn-ghost': selectedCategory !== tab
				})}
				onclick={() => { selectedCategory = tab; page = 0; }}
			>
				{tab}
			</button>
		{/each}
		<input
			type="text"
			class="input input-sm input-bordered w-full max-w-xs"
			placeholder="Search games..."
			bind:value={searchQuery}
		/>
	</div>

	<BrowseGrid
		items={pagedGames}
		{loading}
		{error}
		emptyTitle="No games found"
		onretry={() => fetchGameList(selectedConsoleId)}
		{page}
		{totalPages}
		onpage={handlePageChange}
	>
		{#snippet card(item)}
			{@const game = item as RaGameMetadata}
			<GameCard
				{game}
				onselect={handleSelectGame}
			/>
		{/snippet}
	</BrowseGrid>
</div>
