<script lang="ts">
	import TmdbBrowseCard from './TmdbBrowseCard.svelte';
	import TmdbPagination from './TmdbPagination.svelte';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';

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
		onselectMovie,
		onselectTvShow,
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
		onselectMovie?: (movie: DisplayTMDBMovie) => void;
		onselectTvShow?: (tvShow: DisplayTMDBTvShow) => void;
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
	let results = $derived(isMovies ? movies : tvShows);
	let loading = $derived(isMovies ? loadingMovies : loadingTv);
	let page = $derived(isMovies ? moviesPage : tvPage);
	let totalPages = $derived(isMovies ? moviesTotalPages : tvTotalPages);
	let hasSearched = $derived(query.length > 0);
</script>

<form class="mb-4 flex gap-2" onsubmit={handleSubmit}>
	<input
		type="text"
		class="input input-bordered input-sm flex-1"
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
	<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
		{#if isMovies}
			{#each movies as movie (movie.id)}
				<TmdbBrowseCard {movie} selected={selectedMovieId === movie.id} onclick={onselectMovie ? () => onselectMovie(movie) : undefined} />
			{/each}
		{:else}
			{#each tvShows as tvShow (tvShow.id)}
				<TmdbBrowseCard
					{tvShow}
					selected={selectedTvShowId === tvShow.id}
					onclick={onselectTvShow ? () => onselectTvShow(tvShow) : undefined}
				/>
			{/each}
		{/if}
	</div>
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
