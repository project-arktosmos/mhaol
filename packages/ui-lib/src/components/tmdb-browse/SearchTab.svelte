<script lang="ts">
	import TmdbBrowseGrid from './TmdbBrowseGrid.svelte';
	import TmdbPagination from './TmdbPagination.svelte';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';
	import type { TorrentState } from 'ui-lib/types/torrent.type';

	let {
		movies,
		tvShows,
		moviesPage,
		tvPage,
		moviesTotalPages,
		tvTotalPages,
		query,
		loadingMovies = false,
		loadingTv = false,
		error = null,
		mediaType,
		selectedMovieId = null,
		selectedTvShowId = null,
		fetchedIds,
		downloadStatuses,
		fetchCacheSummaries,
		smartSearchingId = null,
		onselectMovie,
		onselectTvShow,
		onsmartSearch,
		onsearchMovies,
		onsearchTv
	}: {
		movies: DisplayTMDBMovie[];
		tvShows: DisplayTMDBTvShow[];
		moviesPage: number;
		tvPage: number;
		moviesTotalPages: number;
		tvTotalPages: number;
		query: string;
		loadingMovies?: boolean;
		loadingTv?: boolean;
		error?: string | null;
		mediaType?: 'movies' | 'tv';
		selectedMovieId?: number | null;
		selectedTvShowId?: number | null;
		fetchedIds?: Set<number>;
		downloadStatuses?: Map<number, { state: TorrentState; progress: number }>;
		fetchCacheSummaries?: Map<number, string>;
		smartSearchingId?: number | null;
		onselectMovie?: (movie: DisplayTMDBMovie) => void;
		onselectTvShow?: (tvShow: DisplayTMDBTvShow) => void;
		onsmartSearch?: (movie: DisplayTMDBMovie) => void;
		onsearchMovies: (query: string, page: number) => void;
		onsearchTv: (query: string, page: number) => void;
	} = $props();

	let inputValue = $state(query);

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (!inputValue.trim()) return;
		if (mediaType === 'tv') {
			onsearchTv(inputValue, 1);
		} else {
			onsearchMovies(inputValue, 1);
		}
	}

	let isMovies = $derived(mediaType !== 'tv');
	let loading = $derived(isMovies ? loadingMovies : loadingTv);
	let results = $derived(isMovies ? movies : tvShows);
	let page = $derived(isMovies ? moviesPage : tvPage);
	let totalPages = $derived(isMovies ? moviesTotalPages : tvTotalPages);
	let hasSearched = $derived(query.length > 0);
</script>

<form class="mb-4 flex gap-2" onsubmit={handleSubmit}>
	<input
		type="text"
		class="input-bordered input input-sm flex-1"
		placeholder="Search {mediaType === 'tv' ? 'TV shows' : 'movies'}..."
		bind:value={inputValue}
	/>
	<button class="btn btn-sm btn-primary" type="submit" disabled={!inputValue.trim() || loading}>
		{#if loading}
			<span class="loading loading-xs loading-spinner"></span>
		{:else}
			Search
		{/if}
	</button>
</form>

{#if loading}
	<div class="flex justify-center p-8">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else if results.length > 0}
	{#if isMovies}
		<TmdbBrowseGrid {movies} {selectedMovieId} {fetchedIds} {downloadStatuses} {fetchCacheSummaries} {smartSearchingId} {onselectMovie} {onsmartSearch} />
	{:else}
		<TmdbBrowseGrid {tvShows} {selectedTvShowId} {onselectTvShow} />
	{/if}
	<TmdbPagination
		{page}
		{totalPages}
		{loading}
		onpage={(p) => {
			if (isMovies) {
				onsearchMovies(query, p);
			} else {
				onsearchTv(query, p);
			}
		}}
	/>
{:else if error}
	<div class="alert alert-error">
		<p>{error}</p>
	</div>
{:else if hasSearched}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No {mediaType === 'tv' ? 'TV shows' : 'movies'} found for "{query}".</p>
	</div>
{:else}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">
			Enter a search term to find {mediaType === 'tv' ? 'TV shows' : 'movies'}.
		</p>
	</div>
{/if}
