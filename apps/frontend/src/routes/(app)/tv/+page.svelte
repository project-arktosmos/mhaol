<script lang="ts">
	import { onMount } from 'svelte';
	import { goto, invalidateAll } from '$app/navigation';
	import { base } from '$app/paths';
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import type { DisplayTMDBTvShow, TMDBTvShow } from 'addons/tmdb/types';
	import { fetchJson } from 'ui-lib/transport/fetch-helpers';
	import { tvShowsToDisplay } from 'addons/tmdb/transform';
	import { tvShowToDisplay } from 'addons/tmdb/transform';
	import type { MediaCategory, MediaLinkSource } from 'ui-lib/types/media-card.type';
	import type { MediaList } from 'ui-lib/types/media-list.type';
	import TmdbCatalogGrid from 'ui-lib/components/catalog/TmdbCatalogGrid.svelte';
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

	// === Browse state (inline, replaces tmdbBrowseService) ===
	interface TmdbTvPagedResponse { results: TMDBTvShow[]; total_pages: number; page: number; }
	let popularTv = $state<DisplayTMDBTvShow[]>([]);
	let popularTvPage = $state(1);
	let popularTvTotalPages = $state(1);
	let discoverTv = $state<DisplayTMDBTvShow[]>([]);
	let discoverTvPage = $state(1);
	let discoverTvTotalPages = $state(1);
	let tvSelectedGenreId = $state<number | null>(null);
	let tvGenres = $state<Array<{ id: number; name: string }>>([]);
	let searchTv = $state<DisplayTMDBTvShow[]>([]);
	let searchTvPage = $state(1);
	let searchTvTotalPages = $state(1);
	let tvSearchQuery = $state('');
	let tvBrowseLoading = $state<Record<string, boolean>>({});

	async function loadPopularTv(page = 1) {
		tvBrowseLoading = { ...tvBrowseLoading, popularTv: true };
		try {
			const data = await fetchJson<TmdbTvPagedResponse>(`/api/tmdb/popular/tv?page=${page}`);
			popularTv = tvShowsToDisplay(data.results);
			popularTvPage = data.page;
			popularTvTotalPages = data.total_pages;
		} catch { /* best-effort */ }
		tvBrowseLoading = { ...tvBrowseLoading, popularTv: false };
	}

	async function loadDiscoverTv(page = 1, genreId: number | null = null) {
		tvBrowseLoading = { ...tvBrowseLoading, discoverTv: true };
		tvSelectedGenreId = genreId;
		try {
			let url = `/api/tmdb/discover/tv?page=${page}`;
			if (genreId) url += `&with_genres=${genreId}`;
			const data = await fetchJson<TmdbTvPagedResponse>(url);
			discoverTv = tvShowsToDisplay(data.results);
			discoverTvPage = data.page;
			discoverTvTotalPages = data.total_pages;
		} catch { /* best-effort */ }
		tvBrowseLoading = { ...tvBrowseLoading, discoverTv: false };
	}

	async function loadTvGenres() {
		try {
			const data = await fetchJson<{ genres: Array<{ id: number; name: string }> }>('/api/tmdb/genres/tv');
			tvGenres = data?.genres ?? [];
		} catch { /* best-effort */ }
	}

	async function doSearchTv(query: string, page = 1) {
		if (!query.trim()) return;
		tvBrowseLoading = { ...tvBrowseLoading, searchTv: true };
		tvSearchQuery = query;
		try {
			const data = await fetchJson<TmdbTvPagedResponse>(`/api/tmdb/search/tv?q=${encodeURIComponent(query)}&page=${page}`);
			searchTv = tvShowsToDisplay(data.results);
			searchTvPage = data.page;
			searchTvTotalPages = data.total_pages;
		} catch { /* best-effort */ }
		tvBrowseLoading = { ...tvBrowseLoading, searchTv: false };
	}

	const favState = favoritesService.state;
	const pinState = pinsService.state;

	let favoritedTmdbTvIds = $derived(
		new Set($favState.items.filter((f) => f.service === 'tmdb-tv').map((f) => Number(f.serviceId)))
	);
	let pinnedTmdbTvIds = $derived(
		new Set($pinState.items.filter((p) => p.service === 'tmdb-tv').map((p) => Number(p.serviceId)))
	);

	let pinnedTvShows = $state<DisplayTMDBTvShow[]>([]);
	let favoritedTvShows = $state<DisplayTMDBTvShow[]>([]);
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

	async function resolveTvShowIds(ids: number[]): Promise<DisplayTMDBTvShow[]> {
		if (ids.length === 0) return [];
		const results = await Promise.allSettled(
			ids.map((id) => fetchJson<TMDBTvShow>(`/api/tmdb/tv/${id}`))
		);
		return tvShowsToDisplay(
			results
				.filter((r): r is PromiseFulfilledResult<TMDBTvShow> => r.status === 'fulfilled' && r.value != null)
				.map((r) => r.value)
		);
	}

	$effect(() => {
		const ids = [...pinnedTmdbTvIds];
		let cancelled = false;
		resolveTvShowIds(ids).then((shows) => { if (!cancelled) pinnedTvShows = shows; });
		return () => { cancelled = true; };
	});

	$effect(() => {
		const ids = [...favoritedTmdbTvIds];
		let cancelled = false;
		resolveTvShowIds(ids).then((shows) => { if (!cancelled) favoritedTvShows = shows; });
		return () => { cancelled = true; };
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
		loadPopularTv();
		loadTvGenres();
		loadDiscoverTv();
		fetchTmdbMetadataForLists();
	});
</script>

<div class="relative min-w-0 flex-1 overflow-y-auto">
	<div class="flex items-center justify-between gap-4 border-b border-base-300 px-4 py-3">
		<h1 class="text-lg font-bold">TV Shows</h1>
		<div class="flex items-center gap-2">
			<form class="join" onsubmit={(e) => { e.preventDefault(); if (tvSearchInput.trim()) doSearchTv(tvSearchInput.trim()); }}>
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

		{#if favoritedTvShows.length > 0}
			<section class="mb-8">
				<h2 class="mb-3 text-lg font-semibold">Favorites</h2>
				<TmdbCatalogGrid
					tvShows={favoritedTvShows}
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

		{#if tvBrowseLoading['searchTv']}
			<section class="mb-8">
				<h2 class="mb-3 text-lg font-semibold">Search Results</h2>
				<div class="flex justify-center p-8"><span class="loading loading-lg loading-spinner"></span></div>
			</section>
		{:else if searchTv.length > 0}
			<section class="mb-8">
				<h2 class="mb-3 text-lg font-semibold">Search Results</h2>
				<TmdbCatalogGrid
					tvShows={searchTv}
					favoritedIds={favoritedTmdbTvIds}
					pinnedIds={pinnedTmdbTvIds}
					onselectTvShow={handleSelectTvShow}
				/>
				{#if searchTvTotalPages > 1}
					<div class="mt-4 flex items-center justify-center gap-2">
						<button class="btn btn-ghost btn-sm" disabled={searchTvPage <= 1} onclick={() => doSearchTv(tvSearchQuery, searchTvPage - 1)}>Prev</button>
						<span class="text-sm opacity-60">{searchTvPage} / {searchTvTotalPages}</span>
						<button class="btn btn-ghost btn-sm" disabled={searchTvPage >= searchTvTotalPages} onclick={() => doSearchTv(tvSearchQuery, searchTvPage + 1)}>Next</button>
					</div>
				{/if}
			</section>
		{/if}

		<section class="mb-8">
			<h2 class="mb-3 text-lg font-semibold">Popular TV Shows</h2>
			{#if tvBrowseLoading['popularTv']}
				<div class="flex justify-center p-8"><span class="loading loading-lg loading-spinner"></span></div>
			{:else if popularTv.length > 0}
				<TmdbCatalogGrid
					tvShows={popularTv}
					favoritedIds={favoritedTmdbTvIds}
					pinnedIds={pinnedTmdbTvIds}
					onselectTvShow={handleSelectTvShow}
				/>
				{#if popularTvTotalPages > 1}
					<div class="mt-4 flex items-center justify-center gap-2">
						<button class="btn btn-ghost btn-sm" disabled={popularTvPage <= 1} onclick={() => loadPopularTv(popularTvPage - 1)}>Prev</button>
						<span class="text-sm opacity-60">{popularTvPage} / {popularTvTotalPages}</span>
						<button class="btn btn-ghost btn-sm" disabled={popularTvPage >= popularTvTotalPages} onclick={() => loadPopularTv(popularTvPage + 1)}>Next</button>
					</div>
				{/if}
			{/if}
		</section>

		<section class="mb-8">
			<h2 class="mb-3 text-lg font-semibold">Discover TV Shows</h2>
			{#if tvGenres.length > 0}
				<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
					{#each tvGenres as genre (genre.id)}
						<button
							class={classNames('btn btn-sm h-auto min-h-12 flex-col py-2', {
								'btn-primary': tvSelectedGenreId === genre.id,
								'btn-ghost bg-base-200': tvSelectedGenreId !== genre.id
							})}
							onclick={() => {
								const genreId = tvSelectedGenreId === genre.id ? null : genre.id;
								loadDiscoverTv(1, genreId);
							}}
						>
							{genre.name}
						</button>
					{/each}
				</div>
			{/if}
			{#if tvBrowseLoading['discoverTv']}
				<div class="flex justify-center p-8"><span class="loading loading-lg loading-spinner"></span></div>
			{:else if discoverTv.length > 0}
				<div class="mt-4">
					<TmdbCatalogGrid
						tvShows={discoverTv}
						favoritedIds={favoritedTmdbTvIds}
						pinnedIds={pinnedTmdbTvIds}
						onselectTvShow={handleSelectTvShow}
					/>
					{#if discoverTvTotalPages > 1}
						<div class="mt-4 flex items-center justify-center gap-2">
							<button class="btn btn-ghost btn-sm" disabled={discoverTvPage <= 1} onclick={() => loadDiscoverTv(discoverTvPage - 1, tvSelectedGenreId)}>Prev</button>
							<span class="text-sm opacity-60">{discoverTvPage} / {discoverTvTotalPages}</span>
							<button class="btn btn-ghost btn-sm" disabled={discoverTvPage >= discoverTvTotalPages} onclick={() => loadDiscoverTv(discoverTvPage + 1, tvSelectedGenreId)}>Next</button>
						</div>
					{/if}
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
