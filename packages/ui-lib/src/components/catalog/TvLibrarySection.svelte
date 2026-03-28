<script lang="ts">
	import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
	import { tvShowToDisplay, tvShowsToDisplay } from 'addons/tmdb/transform';
	import { fetchJson } from 'ui-lib/transport/fetch-helpers';
	import type { DisplayTMDBTvShow, TMDBTvShow } from 'addons/tmdb/types';
	import type { MediaList } from 'ui-lib/types/media-list.type';
	import { favoritesService } from 'ui-lib/services/favorites.service';
	import { pinsService } from 'ui-lib/services/pins.service';
	import TmdbCatalogGrid from './TmdbCatalogGrid.svelte';
	import TvShowMatchModal from 'ui-lib/components/libraries/TvShowMatchModal.svelte';

	interface AutoMatchResult {
		listId: string;
		matched: boolean;
		tmdbId: number | null;
		tmdbTitle: string | null;
		tmdbYear: string | null;
		tmdbPosterPath: string | null;
		confidence: string;
	}

	interface Props {
		lists: MediaList[];
		libraries: Record<string, { name: string; type: string }>;
		favoritedTmdbTvIds: Set<number>;
		pinnedTmdbTvIds: Set<number>;
		onnavigate: (tmdbId: string) => void;
	}

	let { lists, libraries, favoritedTmdbTvIds, pinnedTmdbTvIds, onnavigate }: Props = $props();

	let matchModalList: MediaList | null = $state(null);
	let tmdbMetadataMap = $state(new Map<string, DisplayTMDBTvShow>());
	let autoMatchingDisplayId: number | null = $state(null);
	let matchAllState: { total: number; completed: number; matched: number } | null = $state(null);

	let tvShowLists = $derived.by(() => {
		const parentTvLists = (lists ?? []).filter(
			(list) => list.parentListId === null && list.libraryType === 'tv'
		);
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
	let listByDisplayId = $derived(
		new Map(tvShowLists.map((list) => [listIdMap.get(list.id)!, list]))
	);

	let seasonCountByShowId = $derived.by(() => {
		const counts = new Map<string, number>();
		for (const list of lists ?? []) {
			if (list.parentListId && list.libraryType === 'tv')
				counts.set(list.parentListId, (counts.get(list.parentListId) ?? 0) + 1);
		}
		return counts;
	});

	let episodeCountByShowId = $derived.by(() => {
		const counts = new Map<string, number>();
		for (const list of lists ?? []) {
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
				...tmdbMeta,
				id: listIdMap.get(list.id)!,
				numberOfSeasons: seasonCount ?? tmdbMeta.numberOfSeasons,
				numberOfEpisodes: episodeCount ?? tmdbMeta.numberOfEpisodes
			};
		}
		return {
			id: listIdMap.get(list.id)!,
			name: list.title,
			originalName: list.title,
			firstAirYear: '',
			lastAirYear: null,
			overview: '',
			posterUrl: null,
			backdropUrl: null,
			voteAverage: 0,
			voteCount: 0,
			genres: [],
			numberOfSeasons: seasonCount,
			numberOfEpisodes: episodeCount
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
			.map(([libraryId, groupLists]) => ({
				libraryId,
				name: libraries[libraryId]?.name ?? libraryId,
				tvShows: groupLists.map(listToDisplayTvShow)
			}))
			.filter((g) => g.tvShows.length > 0);
	});

	async function autoMatchSingle(list: MediaList): Promise<AutoMatchResult | null> {
		const res = await fetchRaw('/api/media-lists/auto-match', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				lists: [{ listId: list.id, title: list.title }]
			})
		});
		if (!res.ok) return null;
		const text = await res.text();
		const line = text.trim().split('\n')[0];
		return line ? (JSON.parse(line) as AutoMatchResult) : null;
	}

	async function handleSelectLibraryShow(tvShow: DisplayTMDBTvShow) {
		const list = listByDisplayId.get(tvShow.id);
		if (!list) return;
		const tmdbLink = list.links?.tmdb;
		if (tmdbLink) {
			onnavigate(tmdbLink.serviceId);
			return;
		}
		autoMatchingDisplayId = tvShow.id;
		try {
			const result = await autoMatchSingle(list);
			if (result?.matched && result.tmdbId) {
				await fetchTmdbMetadataForLists();
				onnavigate(String(result.tmdbId));
			} else {
				matchModalList = list;
			}
		} catch {
			matchModalList = list;
		} finally {
			autoMatchingDisplayId = null;
		}
	}

	async function handleMatchAll(libraryId: string) {
		const unlinked = tvShowLists.filter((l) => l.libraryId === libraryId && !l.links?.tmdb);
		if (unlinked.length === 0) return;
		matchAllState = { total: unlinked.length, completed: 0, matched: 0 };
		try {
			const res = await fetchRaw('/api/media-lists/auto-match', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					lists: unlinked.map((l) => ({ listId: l.id, title: l.title }))
				})
			});
			if (!res.ok) {
				matchAllState = null;
				return;
			}
			const reader = res.body?.getReader();
			if (!reader) {
				matchAllState = null;
				return;
			}
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
					} catch {
						/* skip */
					}
				}
			}
			await fetchTmdbMetadataForLists();
		} finally {
			setTimeout(() => {
				matchAllState = null;
			}, 3000);
		}
	}

	async function handleMatch(tmdbId: number) {
		if (!matchModalList) return;
		const res = await fetchRaw(`/api/media-lists/${matchModalList.id}/tmdb`, {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ tmdbId })
		});
		if (res.ok) {
			matchModalList = null;
			await fetchTmdbMetadataForLists();
		}
	}

	async function fetchTmdbMetadataForLists() {
		const linked = tvShowLists.filter((l) => l.links?.tmdb?.serviceId);
		if (linked.length === 0) return;
		const results = await Promise.all(
			linked.map(async (list) => {
				try {
					const res = await fetchRaw(`/api/tmdb/tv/${list.links.tmdb.serviceId}`);
					if (res.ok)
						return {
							listId: list.id,
							display: tvShowToDisplay(await res.json())
						};
				} catch {
					/* best-effort */
				}
				return null;
			})
		);
		const newMap = new Map(tmdbMetadataMap);
		for (const r of results) {
			if (r) newMap.set(r.listId, r.display);
		}
		tmdbMetadataMap = newMap;
	}

	$effect(() => {
		if (tvShowLists.length > 0) fetchTmdbMetadataForLists();
	});
</script>

{#each libraryGroups as group (group.libraryId)}
	<section class="mb-8 px-4">
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

{#if matchModalList}
	<TvShowMatchModal
		showName={matchModalList.title}
		onmatch={handleMatch}
		onclose={() => (matchModalList = null)}
	/>
{/if}
