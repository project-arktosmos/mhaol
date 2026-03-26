<script lang="ts">
	import { onMount } from 'svelte';
	import { goto, invalidateAll } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { tmdbBrowseService } from 'ui-lib/services/tmdb-browse.service';
	import { smartPairService } from 'ui-lib/services/smart-pair.service';
	import type { DisplayTMDBTvShow } from 'addons/tmdb/types';
	import { tvShowToDisplay } from 'addons/tmdb/transform';
	import type { MediaCategory, MediaLinkSource } from 'ui-lib/types/media-card.type';
	import type { MediaList } from 'ui-lib/types/media-list.type';
	import TmdbCatalogGrid from 'ui-lib/components/catalog/TmdbCatalogGrid.svelte';
	import TmdbPagination from 'ui-lib/components/tmdb-browse/TmdbPagination.svelte';
	import BrowseViewToggle from 'ui-lib/components/browse/BrowseViewToggle.svelte';
	import TvShowMatchModal from 'ui-lib/components/libraries/TvShowMatchModal.svelte';
	import classNames from 'classnames';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';

	interface Props {
		data: {
			mediaTypes: Array<{ id: string; label: string }>;
			categories: MediaCategory[];
			linkSources: MediaLinkSource[];
			lists: MediaList[];
			libraries: Record<string, { name: string; type: string }>;
			error?: string;
		};
	}

	let { data }: Props = $props();

	const browseState = tmdbBrowseService.state;
	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let favoritedTmdbTvIds = $derived(
		new Set($favState.items.filter((f) => f.service === 'tmdb-tv').map((f) => Number(f.serviceId)))
	);
	let pinnedTmdbTvIds = $derived(
		new Set($pinState.items.filter((p) => p.service === 'tmdb-tv').map((p) => Number(p.serviceId)))
	);

	let pinnedTvShows = $state<DisplayTMDBTvShow[]>([]);
	let matchModalList: MediaList | null = $state(null);
	let tvSearchInput = $state('');
	let tmdbMetadataMap = $state(new Map<string, DisplayTMDBTvShow>());
	let autoMatchingDisplayId: number | null = $state(null);
	let matchAllState: { total: number; completed: number; matched: number } | null = $state(null);

	function handleSelectTvShow(tvShow: DisplayTMDBTvShow) {
		goto(`${base}/tv/${tvShow.id}`);
	}

	async function handleSelectLibraryShow(tvShow: DisplayTMDBTvShow) {
		const list = listByDisplayId.get(tvShow.id);
		if (!list) return;
		const tmdbLink = list.links?.tmdb;
		if (tmdbLink) { goto(`${base}/tv/${tmdbLink.serviceId}`); return; }
		autoMatchingDisplayId = tvShow.id;
		try {
			const result = await autoMatchSingle(list);
			if (result?.matched && result.tmdbId) {
				await invalidateAll();
				await fetchTmdbMetadataForLists();
				goto(`${base}/tv/${result.tmdbId}`);
			} else {
				matchModalList = list;
			}
		} catch { matchModalList = list; } finally { autoMatchingDisplayId = null; }
	}

	interface AutoMatchResult {
		listId: string; matched: boolean; tmdbId: number | null;
		tmdbTitle: string | null; tmdbYear: string | null;
		tmdbPosterPath: string | null; confidence: string;
	}

	async function autoMatchSingle(list: MediaList): Promise<AutoMatchResult | null> {
		const res = await fetchRaw('/api/media-lists/auto-match', {
			method: 'POST', headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ lists: [{ listId: list.id, title: list.title }] })
		});
		if (!res.ok) return null;
		const text = await res.text();
		const line = text.trim().split('\n')[0];
		return line ? JSON.parse(line) as AutoMatchResult : null;
	}

	async function handleMatchAll(libraryId: string) {
		const unlinked = tvShowLists.filter((l) => l.libraryId === libraryId && !l.links?.tmdb);
		if (unlinked.length === 0) return;
		matchAllState = { total: unlinked.length, completed: 0, matched: 0 };
		try {
			const res = await fetchRaw('/api/media-lists/auto-match', {
				method: 'POST', headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ lists: unlinked.map((l) => ({ listId: l.id, title: l.title })) })
			});
			if (!res.ok) { matchAllState = null; return; }
			const reader = res.body?.getReader();
			if (!reader) { matchAllState = null; return; }
			const decoder = new TextDecoder();
			let buffer = '';
			while (true) {
				const { done, value } = await reader.read();
				if (done) break;
				buffer += decoder.decode(value, { stream: true });
				const lines = buffer.split('\n');
				buffer = lines.pop() ?? '';
				for (const line of lines) {
					if (!line.trim()) continue;
					try {
						const result: AutoMatchResult = JSON.parse(line);
						matchAllState = {
							total: matchAllState!.total,
							completed: matchAllState!.completed + 1,
							matched: matchAllState!.matched + (result.matched ? 1 : 0)
						};
					} catch { /* skip */ }
				}
			}
			await invalidateAll();
			await fetchTmdbMetadataForLists();
		} finally { setTimeout(() => { matchAllState = null; }, 3000); }
	}

	async function handleMatch(tmdbId: number) {
		if (!matchModalList) return;
		const res = await fetchRaw(`/api/media-lists/${matchModalList.id}/tmdb`, {
			method: 'PUT', headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ tmdbId })
		});
		if (res.ok) {
			matchModalList = null;
			await invalidateAll();
			await fetchTmdbMetadataForLists();
		}
	}

	let tvShowLists = $derived.by(() => {
		const parentTvLists = (data.lists ?? []).filter((list) => list.parentListId === null && list.libraryType === 'tv');
		const seenTmdbIds = new Set<string>();
		return parentTvLists.filter((list) => {
			const tmdbId = list.links?.tmdb?.serviceId;
			if (!tmdbId) return true;
			if (seenTmdbIds.has(tmdbId)) return false;
			seenTmdbIds.add(tmdbId);
			return true;
		});
	});

	let listIdMap = $derived(new Map(tvShowLists.map((list, i) => [list.id, -(i + 1)])));
	let listByDisplayId = $derived(new Map(tvShowLists.map((list) => [listIdMap.get(list.id)!, list])));

	let seasonCountByShowId = $derived.by(() => {
		const counts = new Map<string, number>();
		for (const list of data.lists ?? []) {
			if (list.parentListId && list.libraryType === 'tv')
				counts.set(list.parentListId, (counts.get(list.parentListId) ?? 0) + 1);
		}
		return counts;
	});

	let episodeCountByShowId = $derived.by(() => {
		const counts = new Map<string, number>();
		for (const list of data.lists ?? []) {
			if (list.parentListId && list.libraryType === 'tv')
				counts.set(list.parentListId, (counts.get(list.parentListId) ?? 0) + list.itemCount);
		}
		return counts;
	});

	let unlinkedCountByLibrary = $derived.by(() => {
		const counts = new Map<string, number>();
		for (const list of tvShowLists) {
			if (!list.links?.tmdb) counts.set(list.libraryId, (counts.get(list.libraryId) ?? 0) + 1);
		}
		return counts;
	});

	function listToDisplayTvShow(list: MediaList): DisplayTMDBTvShow {
		const seasonCount = seasonCountByShowId.get(list.id) ?? null;
		const episodeCount = episodeCountByShowId.get(list.id) ?? list.itemCount ?? null;
		const tmdbMeta = tmdbMetadataMap.get(list.id);
		if (tmdbMeta) {
			return {
				...tmdbMeta, id: listIdMap.get(list.id)!,
				numberOfSeasons: seasonCount ?? tmdbMeta.numberOfSeasons,
				numberOfEpisodes: episodeCount ?? tmdbMeta.numberOfEpisodes
			};
		}
		return {
			id: listIdMap.get(list.id)!, name: list.title, originalName: list.title,
			firstAirYear: '', lastAirYear: null, overview: '',
			posterUrl: null, backdropUrl: null, voteAverage: 0, voteCount: 0, genres: [],
			numberOfSeasons: seasonCount, numberOfEpisodes: episodeCount
		};
	}

	let libraryGroups = $derived.by(() => {
		const grouped = new Map<string, MediaList[]>();
		for (const list of tvShowLists) {
			const existing = grouped.get(list.libraryId);
			if (existing) existing.push(list);
			else grouped.set(list.libraryId, [list]);
		}
		return Array.from(grouped.entries())
			.map(([libraryId, lists]) => ({
				libraryId, name: data.libraries[libraryId]?.name ?? libraryId,
				tvShows: lists.map(listToDisplayTvShow)
			}))
			.filter((g) => g.tvShows.length > 0);
	});

	async function fetchTmdbMetadataForLists() {
		const linked = tvShowLists.filter((l) => l.links?.tmdb?.serviceId);
		if (linked.length === 0) return;
		const results = await Promise.all(
			linked.map(async (list) => {
				try {
					const res = await fetchRaw(`/api/tmdb/tv/${list.links.tmdb.serviceId}`);
					if (res.ok) return { listId: list.id, display: tvShowToDisplay(await res.json()) };
				} catch { /* best-effort */ }
				return null;
			})
		);
		const newMap = new Map(tmdbMetadataMap);
		for (const r of results) { if (r) newMap.set(r.listId, r.display); }
		tmdbMetadataMap = newMap;
	}

	onMount(() => {
		tmdbBrowseService.loadPopularTv();
		tmdbBrowseService.loadGenres();
		tmdbBrowseService.loadDiscoverTv();
		smartPairService.loadPinned().then((pinned) => { pinnedTvShows = pinned.tv; });
		fetchTmdbMetadataForLists();
	});
</script>

<div class="relative min-w-0 flex-1 overflow-y-auto">
	<div class="flex items-center justify-between gap-4 border-b border-base-300 px-4 py-3">
		<h1 class="text-lg font-bold">TV Shows</h1>
		<div class="flex items-center gap-2">
			<form class="join" onsubmit={(e) => { e.preventDefault(); if (tvSearchInput.trim()) tmdbBrowseService.searchTv(tvSearchInput.trim()); }}>
				<input type="text" placeholder="Search TV shows..." class="input join-item input-bordered input-sm w-48" bind:value={tvSearchInput} />
				<button type="submit" class="btn join-item btn-sm btn-primary">Search</button>
			</form>
			<BrowseViewToggle />
		</div>
	</div>
	<div class="container mx-auto p-4">
		{#if pinnedTvShows.length > 0}
			<section class="mb-8">
				<h2 class="mb-3 text-lg font-semibold">Pinned</h2>
				<TmdbCatalogGrid
					tvShows={pinnedTvShows}
					favoritedIds={favoritedTmdbTvIds}
					pinnedIds={pinnedTmdbTvIds}
					onselectTvShow={handleSelectTvShow}
				/>
			</section>
		{/if}

		{#each libraryGroups as group (group.libraryId)}
			<section class="mb-8">
				<div class="mb-3 flex items-center gap-3">
					<h2 class="text-lg font-semibold">{group.name}</h2>
					{#if (unlinkedCountByLibrary.get(group.libraryId) ?? 0) > 0}
						{#if matchAllState}
							<span class="text-sm opacity-70">
								{#if matchAllState.completed < matchAllState.total}
									Matching {matchAllState.completed}/{matchAllState.total}...
									<span class="loading loading-xs loading-spinner"></span>
								{:else}
									Matched {matchAllState.matched}/{matchAllState.total}
								{/if}
							</span>
						{:else}
							<button class="btn btn-outline btn-xs" onclick={() => handleMatchAll(group.libraryId)}>
								Match All ({unlinkedCountByLibrary.get(group.libraryId)})
							</button>
						{/if}
					{/if}
				</div>
				<TmdbCatalogGrid
					tvShows={group.tvShows}
					favoritedIds={favoritedTmdbTvIds}
					pinnedIds={pinnedTmdbTvIds}
					matchingTvShowId={autoMatchingDisplayId}
					onselectTvShow={handleSelectLibraryShow}
				/>
			</section>
		{/each}

		{#if $browseState.loading['searchTv']}
			<section class="mb-8">
				<h2 class="mb-3 text-lg font-semibold">Search Results</h2>
				<div class="flex justify-center p-8"><span class="loading loading-lg loading-spinner"></span></div>
			</section>
		{:else if $browseState.searchTv.length > 0}
			<section class="mb-8">
				<h2 class="mb-3 text-lg font-semibold">Search Results</h2>
				<TmdbCatalogGrid
					tvShows={$browseState.searchTv}
					favoritedIds={favoritedTmdbTvIds}
					pinnedIds={pinnedTmdbTvIds}
					onselectTvShow={handleSelectTvShow}
				/>
				<TmdbPagination
					page={$browseState.searchTvPage}
					totalPages={$browseState.searchTvTotalPages}
					loading={$browseState.loading['searchTv'] ?? false}
					onpage={(p) => tmdbBrowseService.searchTv($browseState.searchQuery, p)}
				/>
			</section>
		{/if}

		<section class="mb-8">
			<h2 class="mb-3 text-lg font-semibold">Popular TV Shows</h2>
			{#if $browseState.loading['popularTv']}
				<div class="flex justify-center p-8"><span class="loading loading-lg loading-spinner"></span></div>
			{:else if $browseState.popularTv.length > 0}
				<TmdbCatalogGrid
					tvShows={$browseState.popularTv}
					favoritedIds={favoritedTmdbTvIds}
					pinnedIds={pinnedTmdbTvIds}
					onselectTvShow={handleSelectTvShow}
				/>
				<TmdbPagination
					page={$browseState.popularTvPage}
					totalPages={$browseState.popularTvTotalPages}
					loading={$browseState.loading['popularTv'] ?? false}
					onpage={(p) => tmdbBrowseService.loadPopularTv(p)}
				/>
			{/if}
		</section>

		<section class="mb-8">
			<h2 class="mb-3 text-lg font-semibold">Discover TV Shows</h2>
			{#if $browseState.tvGenres.length > 0}
				<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
					{#each $browseState.tvGenres as genre (genre.id)}
						<button
							class={classNames('btn btn-sm h-auto min-h-12 flex-col py-2', {
								'btn-primary': $browseState.selectedGenreId === genre.id,
								'btn-ghost bg-base-200': $browseState.selectedGenreId !== genre.id
							})}
							onclick={() => {
								const genreId = $browseState.selectedGenreId === genre.id ? null : genre.id;
								tmdbBrowseService.loadDiscoverTv(1, genreId);
							}}
						>
							{genre.name}
						</button>
					{/each}
				</div>
			{/if}
			{#if $browseState.loading['discoverTv']}
				<div class="flex justify-center p-8"><span class="loading loading-lg loading-spinner"></span></div>
			{:else if $browseState.discoverTv.length > 0}
				<div class="mt-4">
					<TmdbCatalogGrid
						tvShows={$browseState.discoverTv}
						favoritedIds={favoritedTmdbTvIds}
						pinnedIds={pinnedTmdbTvIds}
						onselectTvShow={handleSelectTvShow}
					/>
					<TmdbPagination
						page={$browseState.discoverTvPage}
						totalPages={$browseState.discoverTvTotalPages}
						loading={$browseState.loading['discoverTv'] ?? false}
						onpage={(p) => tmdbBrowseService.loadDiscoverTv(p, $browseState.selectedGenreId)}
					/>
				</div>
			{/if}
		</section>
	</div>
</div>

{#if matchModalList}
	<TvShowMatchModal
		showName={matchModalList.title}
		onmatch={handleMatch}
		onclose={() => (matchModalList = null)}
	/>
{/if}
