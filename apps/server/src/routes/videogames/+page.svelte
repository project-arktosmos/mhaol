<script lang="ts">
	import { onMount } from 'svelte';
	import { apiUrl } from 'frontend/lib/api-base';
	import { retroAchievementsAdapter } from 'frontend/adapters/classes/retroachievements.adapter';
	import type { RaGameMetadata } from 'frontend/types/retroachievements.type';
	import { RA_CONSOLES } from 'frontend/types/retroachievements.type';
	import GameCard from 'ui-lib/components/videogames/GameCard.svelte';
	import classNames from 'classnames';

	const PAGE_SIZE = 20;

	let selectedConsoleId = $state(5);
	let games = $state<RaGameMetadata[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let selectedGame = $state<RaGameMetadata | null>(null);
	let page = $state(0);

	let gameDetails = $state<RaGameMetadata | null>(null);
	let detailsLoading = $state(false);

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
		// Originals first, then alphabetical
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

	// Track which backfill run is current so stale ones abort
	let backfillGeneration = 0;

	async function fetchGameList(consoleId: number) {
		if (consoleCache[consoleId]) {
			games = consoleCache[consoleId];
			return;
		}

		loading = true;
		error = null;
		try {
			const res = await fetch(
				apiUrl(`/api/retroachievements/games?console=${consoleId}`)
			);
			if (!res.ok) throw new Error('Failed to fetch game list');
			const data = await res.json();
			if (!Array.isArray(data)) throw new Error('Unexpected response format');
			const display = data.map((g: unknown) =>
				retroAchievementsAdapter.fromGameListItem(
					g as Parameters<typeof retroAchievementsAdapter.fromGameListItem>[0]
				)
			);
			consoleCache[consoleId] = display;
			games = display;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
			games = [];
		}
		loading = false;
	}

	// Backfill box art for the current page only
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
						const detail = retroAchievementsAdapter.fromGameExtended(
							data as Parameters<typeof retroAchievementsAdapter.fromGameExtended>[0]
						);
						detailsCache[game.id] = detail;
						game.imageBoxArtUrl = detail.imageBoxArtUrl;
						game.imageTitleUrl = detail.imageTitleUrl;
						game.imageIngameUrl = detail.imageIngameUrl;
					} catch {
						// skip
					}
				})
			);
			if (backfillGeneration !== gen) return;
			games = [...games];
		}
	}

	// Re-backfill whenever the visible page changes
	$effect(() => {
		// Access pagedGames to track dependency
		if (pagedGames.length > 0 && !loading) {
			backfillPageImages();
		}
	});

	async function fetchGameDetails(gameId: number) {
		if (detailsCache[gameId]) {
			gameDetails = detailsCache[gameId];
			return;
		}

		detailsLoading = true;
		gameDetails = null;
		try {
			const res = await fetch(apiUrl(`/api/retroachievements/games/${gameId}`));
			if (!res.ok) throw new Error('Failed to fetch game details');
			const data = await res.json();
			const detail = retroAchievementsAdapter.fromGameExtended(
				data as Parameters<typeof retroAchievementsAdapter.fromGameExtended>[0]
			);
			detailsCache[gameId] = detail;
			gameDetails = detail;
		} catch {
			gameDetails = null;
		}
		detailsLoading = false;
	}

	function handleConsoleChange(consoleId: number) {
		selectedConsoleId = consoleId;
		selectedGame = null;
		gameDetails = null;
		searchQuery = '';
		selectedCategory = 'Originals';
		page = 0;
		fetchGameList(consoleId);
	}

	function handleSelectGame(game: RaGameMetadata) {
		if (selectedGame?.id === game.id) {
			selectedGame = null;
			gameDetails = null;
		} else {
			selectedGame = game;
			gameDetails = null;
			fetchGameDetails(game.id);
		}
	}

	function handlePageChange(newPage: number) {
		page = newPage;
		selectedGame = null;
		gameDetails = null;
	}

	// Reset page when search changes
	$effect(() => {
		searchQuery;
		page = 0;
	});

	onMount(() => {
		fetchGameList(selectedConsoleId);
	});
</script>

<div class="flex h-full w-full">
	<div class="flex flex-1 flex-col overflow-hidden">
		<!-- Header -->
		<div class="flex items-center gap-3 border-b border-base-300 px-4 py-3">
			<h2 class="text-lg font-bold">Videogames</h2>
			<span class="badge badge-ghost">{filteredGames.length} games</span>
		</div>

		<!-- Console tabs -->
		<div class="flex flex-wrap gap-1.5 border-b border-base-300 px-4 py-2">
			{#each RA_CONSOLES as console}
				<button
					class={classNames('btn btn-xs', {
						'btn-primary': selectedConsoleId === console.id,
						'btn-ghost': selectedConsoleId !== console.id
					})}
					onclick={() => handleConsoleChange(console.id)}
				>
					{console.name}
				</button>
			{/each}
		</div>

		<!-- Category tabs + Search -->
		<div class="flex items-center gap-3 border-b border-base-300 px-4 py-2">
			{#each categoryTabs as tab}
				<button
					class={classNames('btn btn-xs', {
						'btn-secondary': selectedCategory === tab,
						'btn-ghost': selectedCategory !== tab
					})}
					onclick={() => {
						selectedCategory = tab;
						page = 0;
					}}
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

		<!-- Grid -->
		<div class="flex-1 overflow-y-auto p-4">
			{#if loading}
				<div class="flex items-center justify-center py-16">
					<span class="loading loading-lg loading-spinner"></span>
				</div>
			{:else if error}
				<div class="flex flex-col items-center justify-center py-16 text-base-content/40">
					<p class="text-lg">Failed to load games</p>
					<p class="mt-1 text-sm">{error}</p>
					<button
						class="btn btn-primary btn-sm mt-4"
						onclick={() => fetchGameList(selectedConsoleId)}
					>
						Retry
					</button>
				</div>
			{:else if pagedGames.length === 0}
				<div class="flex flex-col items-center justify-center py-16 text-base-content/40">
					<p class="text-lg">No games found</p>
				</div>
			{:else}
				<div
					class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6"
				>
					{#each pagedGames as game (game.id)}
						<GameCard
							{game}
							selected={selectedGame?.id === game.id}
							onselect={handleSelectGame}
						/>
					{/each}
				</div>

				<!-- Pagination -->
				{#if totalPages > 1}
					<div class="mt-4 flex items-center justify-center gap-2">
						<button
							class="btn btn-ghost btn-sm"
							disabled={page === 0}
							onclick={() => handlePageChange(page - 1)}
						>
							Prev
						</button>
						<span class="text-sm opacity-60">
							{page + 1} / {totalPages}
						</span>
						<button
							class="btn btn-ghost btn-sm"
							disabled={page >= totalPages - 1}
							onclick={() => handlePageChange(page + 1)}
						>
							Next
						</button>
					</div>
				{/if}
			{/if}
		</div>
	</div>

	<!-- Right sidebar: selected game detail -->
	{#if selectedGame}
		<div class="flex w-80 flex-col gap-3 overflow-y-auto border-l border-base-300 bg-base-100 p-4">
			<div class="flex flex-col gap-2">
				{#if selectedGame.imageIconUrl}
					<img
						src={selectedGame.imageIconUrl}
						alt={selectedGame.title}
						class="aspect-square w-full rounded-lg object-cover"
					/>
				{:else}
					<div
						class="flex aspect-square w-full items-center justify-center rounded-lg bg-base-200"
					>
						<svg
							xmlns="http://www.w3.org/2000/svg"
							class="h-16 w-16 text-base-content/20"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="1.5"
								d="M14.25 6.087c0-.355.186-.676.401-.959.221-.29.349-.634.349-1.003 0-1.036-1.007-1.875-2.25-1.875s-2.25.84-2.25 1.875c0 .369.128.713.349 1.003.215.283.401.604.401.959v0a.64.64 0 01-.657.643 48.39 48.39 0 01-4.163-.3c.186 1.613.293 3.25.315 4.907a.656.656 0 01-.658.663v0c-.355 0-.676-.186-.959-.401a1.647 1.647 0 00-1.003-.349c-1.036 0-1.875 1.007-1.875 2.25s.84 2.25 1.875 2.25c.369 0 .713-.128 1.003-.349.283-.215.604-.401.959-.401v0c.31 0 .555.26.532.57a48.039 48.039 0 01-.642 5.056c1.518.19 3.058.309 4.616.354a.64.64 0 00.657-.643v0c0-.355-.186-.676-.401-.959a1.647 1.647 0 01-.349-1.003c0-1.035 1.008-1.875 2.25-1.875 1.243 0 2.25.84 2.25 1.875 0 .369-.128.713-.349 1.003-.215.283-.4.604-.4.959v0c0 .333.277.599.61.58a48.1 48.1 0 005.427-.63 48.05 48.05 0 00.582-4.717.532.532 0 00-.533-.57v0c-.355 0-.676.186-.959.401-.29.221-.634.349-1.003.349-1.035 0-1.875-1.007-1.875-2.25s.84-2.25 1.875-2.25c.37 0 .713.128 1.003.349.283.215.604.401.959.401v0a.656.656 0 00.658-.663 48.422 48.422 0 00-.37-5.36c-1.886.342-3.81.574-5.766.689a.578.578 0 01-.61-.58v0z"
							/>
						</svg>
					</div>
				{/if}

				<h3 class="text-sm font-bold">{selectedGame.title}</h3>
				<p class="text-xs opacity-60">{selectedGame.consoleName}</p>

				{#if detailsLoading}
					<div class="flex items-center justify-center py-4">
						<span class="loading loading-sm loading-spinner"></span>
					</div>
				{:else if gameDetails}
					<div class="flex flex-col gap-1.5">
						{#if gameDetails.developer}
							<div class="flex items-center gap-1 text-xs">
								<span class="opacity-40">Developer:</span>
								<span>{gameDetails.developer}</span>
							</div>
						{/if}
						{#if gameDetails.publisher}
							<div class="flex items-center gap-1 text-xs">
								<span class="opacity-40">Publisher:</span>
								<span>{gameDetails.publisher}</span>
							</div>
						{/if}
						{#if gameDetails.genre}
							<div class="flex items-center gap-1 text-xs">
								<span class="opacity-40">Genre:</span>
								<span>{gameDetails.genre}</span>
							</div>
						{/if}
						{#if gameDetails.released}
							<div class="flex items-center gap-1 text-xs">
								<span class="opacity-40">Released:</span>
								<span>{gameDetails.released}</span>
							</div>
						{/if}
						{#if gameDetails.numDistinctPlayers}
							<div class="flex items-center gap-1 text-xs">
								<span class="opacity-40">Players:</span>
								<span>{gameDetails.numDistinctPlayers.toLocaleString()}</span>
							</div>
						{/if}

						<div class="flex flex-wrap gap-1 pt-1">
							{#if gameDetails.numAchievements > 0}
								<span class="badge badge-info badge-sm"
									>{gameDetails.numAchievements} achievements</span
								>
							{/if}
							{#if gameDetails.points > 0}
								<span class="badge badge-ghost badge-sm">{gameDetails.points} points</span>
							{/if}
						</div>

						<!-- Game images -->
						{#if gameDetails.imageBoxArtUrl}
							<img
								src={gameDetails.imageBoxArtUrl}
								alt="Box art"
								class="mt-2 w-full rounded-lg"
								loading="lazy"
							/>
						{/if}
						{#if gameDetails.imageIngameUrl}
							<img
								src={gameDetails.imageIngameUrl}
								alt="In-game screenshot"
								class="w-full rounded-lg"
								loading="lazy"
							/>
						{/if}
						{#if gameDetails.imageTitleUrl}
							<img
								src={gameDetails.imageTitleUrl}
								alt="Title screen"
								class="w-full rounded-lg"
								loading="lazy"
							/>
						{/if}
					</div>

					<!-- Achievements list -->
					{#if gameDetails.achievements && gameDetails.achievements.length > 0}
						<div class="flex flex-col gap-0.5 pt-2">
							<div class="flex items-center justify-between">
								<h4 class="text-xs font-semibold opacity-50">Achievements</h4>
								<span class="text-xs opacity-30"
									>{gameDetails.achievements.length} total</span
								>
							</div>
							{#each gameDetails.achievements as achievement (achievement.id)}
								<div class="flex items-center gap-2 rounded px-1 py-1 hover:bg-base-200">
									{#if achievement.badgeUrl}
										<img
											src={achievement.badgeUrl}
											alt={achievement.title}
											class="h-8 w-8 rounded"
											loading="lazy"
										/>
									{/if}
									<div class="min-w-0 flex-1">
										<p class="truncate text-xs font-medium">{achievement.title}</p>
										<p class="truncate text-xs opacity-40">{achievement.description}</p>
									</div>
									<span class="text-xs opacity-30">{achievement.points}pts</span>
								</div>
							{/each}
						</div>
					{/if}
				{/if}
			</div>
		</div>
	{/if}
</div>
