<script lang="ts">
	import TmdbBrowseGrid from './TmdbBrowseGrid.svelte';
	import TmdbPagination from './TmdbPagination.svelte';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';
	import type { TorrentState } from 'ui-lib/types/torrent.type';

	interface LinkedItem {
		tmdbId: number;
		title: string;
		type: 'movie' | 'tv';
	}

	let {
		linkedItems,
		recommendations,
		page,
		totalPages,
		sourceId,
		sourceType,
		loading = false,
		error = null,
		selectedMovieId = null,
		selectedTvShowId = null,
		fetchedIds,
		downloadStatuses,
		onselectMovie,
		onselectTvShow,
		onload
	}: {
		linkedItems: LinkedItem[];
		recommendations: (DisplayTMDBMovie | DisplayTMDBTvShow)[];
		page: number;
		totalPages: number;
		sourceId: number | null;
		sourceType: 'movie' | 'tv' | null;
		loading?: boolean;
		error?: string | null;
		selectedMovieId?: number | null;
		selectedTvShowId?: number | null;
		fetchedIds?: Set<number>;
		downloadStatuses?: Map<number, { state: TorrentState; progress: number }>;
		onselectMovie?: (movie: DisplayTMDBMovie) => void;
		onselectTvShow?: (tvShow: DisplayTMDBTvShow) => void;
		onload: (tmdbId: number, type: 'movie' | 'tv', page: number) => void;
	} = $props();

	let selectedSource = $derived(
		sourceId != null
			? `${sourceType}:${sourceId}`
			: linkedItems.length > 0
				? `${linkedItems[0].type}:${linkedItems[0].tmdbId}`
				: ''
	);

	function handleSourceChange(e: Event) {
		const val = (e.target as HTMLSelectElement).value;
		if (!val) return;
		const [type, id] = val.split(':');
		onload(Number(id), type as 'movie' | 'tv', 1);
	}

	let recMovies = $derived(
		recommendations.filter((r): r is DisplayTMDBMovie => 'title' in r)
	);
	let recTvShows = $derived(
		recommendations.filter((r): r is DisplayTMDBTvShow => !('title' in r))
	);
</script>

{#if linkedItems.length === 0}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">
			Link movies or TV shows to TMDB in your library to get recommendations.
		</p>
	</div>
{:else}
	<div class="mb-4">
		<label class="text-sm font-medium">
			Based on:
			<select
				class="select-bordered select ml-2 select-sm"
				value={selectedSource}
				onchange={handleSourceChange}
			>
				{#each linkedItems as item}
					<option value="{item.type}:{item.tmdbId}">
						{item.title} ({item.type === 'movie' ? 'Movie' : 'TV'})
					</option>
				{/each}
			</select>
		</label>
	</div>

	{#if loading}
		<div class="flex justify-center p-8">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if recommendations.length > 0}
		<TmdbBrowseGrid
			movies={recMovies}
			tvShows={recTvShows}
			{selectedMovieId}
			{selectedTvShowId}
			{fetchedIds}
			{downloadStatuses}
			{onselectMovie}
			{onselectTvShow}
		/>
		{#if sourceId != null && sourceType != null}
			<TmdbPagination
				{page}
				{totalPages}
				{loading}
				onpage={(p) => onload(sourceId!, sourceType!, p)}
			/>
		{/if}
	{:else if error}
		<div class="alert alert-error">
			<p>{error}</p>
		</div>
	{:else if sourceId != null}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No recommendations found for this title.</p>
		</div>
	{:else}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">Select a title above to see recommendations.</p>
		</div>
	{/if}
{/if}
