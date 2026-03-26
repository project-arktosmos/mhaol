<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import { torrentService } from 'ui-lib/services/torrent.service';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import { gameExtendedToDisplay } from 'addons/retroachievements';
	import type { RaGameExtended } from 'addons/retroachievements/types';
	import type { CatalogGame } from 'ui-lib/types/catalog.type';
	import CatalogDetailPage from 'ui-lib/components/catalog/CatalogDetailPage.svelte';
	import GameDetailMeta from 'ui-lib/components/catalog/detail/GameDetailMeta.svelte';

	let catalogItem = $state<CatalogGame | null>(null);
	let loading = $state(true);
	let fetchingGameId = $state<number | null>(null);

	const searchStore = smartSearchService.store;
	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let id = $derived($page.params.id ?? '');
	let gameId = $derived(Number(id));

	let isFavorite = $derived($favState.items.some((f) => f.service === 'retroachievements' && f.serviceId === String(gameId)));
	let isPinned = $derived($pinState.items.some((p) => p.service === 'retroachievements' && p.serviceId === String(gameId)));

	let isFetching = $derived(
		fetchingGameId !== null && fetchingGameId === gameId &&
			$searchStore.fetchedCandidate === null && $searchStore.selection?.mode === 'fetch'
	);
	let isFetchedForCurrent = $derived($searchStore.fetchedCandidate !== null && fetchingGameId === gameId);

	let currentFetchSteps = $derived.by(() => {
		if (!isFetching && !isFetchedForCurrent) return null;
		if (isFetchedForCurrent) return { terms: true, search: true, searching: false, eval: true, done: true };
		const s = $searchStore;
		return {
			terms: s.selection !== null, search: !s.searching && s.searchResults.length > 0,
			searching: s.searching, eval: s.searchResults.some((r) => r.analysis !== null),
			done: s.fetchedCandidate !== null
		};
	});

	let matchedTorrent = $derived.by(() => {
		const candidate = $searchStore.fetchedCandidate;
		if (candidate?.infoHash) { const t = torrentService.findByHash(candidate.infoHash); if (t) return t; }
		return null;
	});

	async function fetchGame(gameId: string) {
		loading = true;
		smartSearchService.clear();
		try {
			const res = await fetchRaw(`/api/retroachievements/games/${gameId}`);
			if (!res.ok) throw new Error('Failed to fetch game');
			const detail = gameExtendedToDisplay(await res.json() as RaGameExtended);
			catalogItem = {
				id: String(detail.id), kind: 'game',
				title: detail.title, sortTitle: detail.title.toLowerCase(),
				year: detail.released ?? null, overview: null,
				posterUrl: detail.imageBoxArtUrl ?? detail.imageIconUrl, backdropUrl: detail.imageTitleUrl ?? null,
				voteAverage: null, voteCount: null,
				parentId: null, position: null,
				source: 'retroachievements', sourceId: String(detail.id),
				createdAt: '', updatedAt: '',
				metadata: {
					retroachievementsId: detail.id, consoleId: detail.consoleId,
					consoleName: detail.consoleName, imageIconUrl: detail.imageIconUrl,
					numAchievements: detail.numAchievements, points: detail.points,
					developer: detail.developer ?? null, publisher: detail.publisher ?? null,
					genre: detail.genre ?? null, released: detail.released ?? null,
					imageTitleUrl: detail.imageTitleUrl ?? null,
					imageIngameUrl: detail.imageIngameUrl ?? null,
					imageBoxArtUrl: detail.imageBoxArtUrl ?? null,
					achievements: (detail.achievements ?? []).map((a) => ({
						id: a.id, title: a.title, description: a.description,
						points: a.points, trueRatio: a.trueRatio, badgeUrl: a.badgeUrl,
						displayOrder: a.displayOrder, numAwarded: a.numAwarded,
						numAwardedHardcore: a.numAwardedHardcore
					}))
				}
			};
		} catch { catalogItem = null; }
		loading = false;
	}

	function handleFetch() {
		if (!catalogItem) return;
		fetchingGameId = catalogItem.metadata.retroachievementsId;
		smartSearchService.select({
			title: catalogItem.title, year: '', type: 'game',
			retroachievementsId: catalogItem.metadata.retroachievementsId,
			consoleName: catalogItem.metadata.consoleName, mode: 'fetch'
		});
	}

	function handleDownload() {
		const candidate = smartSearchService.getFetchedCandidate();
		if (candidate) smartSearchService.startDownload(candidate);
	}

	async function handleToggleFavorite() {
		if (catalogItem) await favoritesService.toggle('retroachievements', catalogItem.sourceId, catalogItem.title);
	}
	async function handleTogglePin() {
		if (catalogItem) await pinsService.toggle('retroachievements', catalogItem.sourceId, catalogItem.title);
	}

	onMount(() => fetchGame(id));
</script>

{#if catalogItem}
	<CatalogDetailPage
		item={catalogItem} {loading}
		fetching={isFetching} fetched={isFetchedForCurrent}
		fetchSteps={currentFetchSteps} torrentStatus={matchedTorrent}
		fetchedTorrent={$searchStore.fetchedCandidate ? { name: $searchStore.fetchedCandidate.name, quality: $searchStore.fetchedCandidate.analysis?.quality ?? '', languages: $searchStore.fetchedCandidate.analysis?.languages ?? '' } : null}
		{isFavorite} {isPinned}
		onfetch={handleFetch} ondownload={handleDownload}
		onshowsearch={() => smartSearchService.show()}
		onback={() => goto(`${base}/videogames`)}
		ontogglefavorite={handleToggleFavorite} ontogglepin={handleTogglePin}
	>
		{#snippet extra()}<GameDetailMeta item={catalogItem!} />{/snippet}
	</CatalogDetailPage>
{:else if loading}
	<div class="flex flex-1 items-center justify-center"><span class="loading loading-lg loading-spinner"></span></div>
{:else}
	<div class="flex flex-1 flex-col items-center justify-center gap-2">
		<p class="text-sm opacity-60">Game not found</p>
		<button class="btn btn-ghost btn-sm" onclick={() => goto(`${base}/videogames`)}>Back to games</button>
	</div>
{/if}
