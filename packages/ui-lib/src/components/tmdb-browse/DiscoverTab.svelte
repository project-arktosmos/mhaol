<script lang="ts">
	import classNames from 'classnames';
	import TmdbBrowseGrid from './TmdbBrowseGrid.svelte';
	import TmdbPagination from './TmdbPagination.svelte';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';
	import type { TmdbGenre } from 'ui-lib/types/tmdb-browse.type';
	import type { TorrentState } from 'ui-lib/types/torrent.type';

	let {
		movies,
		tvShows,
		moviesPage,
		tvPage,
		moviesTotalPages,
		tvTotalPages,
		movieGenres,
		tvGenres,
		selectedGenreId,
		loadingMovies = false,
		loadingTv = false,
		error = null,
		mediaType,
		selectedMovieId = null,
		selectedTvShowId = null,
		fetchedIds,
		favoritedIds,
		pinnedIds,
		downloadStatuses,
		onselectMovie,
		onselectTvShow,
		ondiscoverMovies,
		ondiscoverTv
	}: {
		movies: DisplayTMDBMovie[];
		tvShows: DisplayTMDBTvShow[];
		moviesPage: number;
		tvPage: number;
		moviesTotalPages: number;
		tvTotalPages: number;
		movieGenres: TmdbGenre[];
		tvGenres: TmdbGenre[];
		selectedGenreId: number | null;
		loadingMovies?: boolean;
		loadingTv?: boolean;
		error?: string | null;
		mediaType?: 'movies' | 'tv';
		selectedMovieId?: number | null;
		selectedTvShowId?: number | null;
		fetchedIds?: Set<number>;
		favoritedIds?: Set<number>;
		pinnedIds?: Set<number>;
		downloadStatuses?: Map<number, { state: TorrentState; progress: number }>;
		onselectMovie?: (movie: DisplayTMDBMovie) => void;
		onselectTvShow?: (tvShow: DisplayTMDBTvShow) => void;
		ondiscoverMovies: (page: number, genreId: number | null) => void;
		ondiscoverTv: (page: number, genreId: number | null) => void;
	} = $props();

	let subTabInternal = $state<'movies' | 'tv'>('movies');
	let subTab = $derived(mediaType ?? subTabInternal);

	let genres = $derived(subTab === 'movies' ? movieGenres : tvGenres);

	function handleGenreChange(e: Event) {
		const val = (e.target as HTMLSelectElement).value;
		const genreId = val === '' ? null : Number(val);
		if (subTab === 'movies') {
			ondiscoverMovies(1, genreId);
		} else {
			ondiscoverTv(1, genreId);
		}
	}
</script>

<div class="mb-4 flex flex-wrap items-center gap-2">
	{#if !mediaType}
		<button
			class={classNames('btn btn-xs', {
				'btn-secondary': subTab === 'movies',
				'btn-ghost': subTab !== 'movies'
			})}
			onclick={() => (subTabInternal = 'movies')}
		>
			Movies
		</button>
		<button
			class={classNames('btn btn-xs', {
				'btn-secondary': subTab === 'tv',
				'btn-ghost': subTab !== 'tv'
			})}
			onclick={() => (subTabInternal = 'tv')}
		>
			TV Shows
		</button>
	{/if}

	<select
		class={classNames('select-bordered select select-xs', { 'ml-auto': !mediaType })}
		value={selectedGenreId ?? ''}
		onchange={handleGenreChange}
	>
		<option value="">All Genres</option>
		{#each genres as genre (genre.id)}
			<option value={genre.id}>{genre.name}</option>
		{/each}
	</select>
</div>

{#if subTab === 'movies'}
	{#if loadingMovies}
		<div class="flex justify-center p-8">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if movies.length > 0}
		<TmdbBrowseGrid
			{movies}
			{selectedMovieId}
			{fetchedIds}
			{favoritedIds}
			{pinnedIds}
			{downloadStatuses}
			{onselectMovie}
		/>
		<TmdbPagination
			page={moviesPage}
			totalPages={moviesTotalPages}
			loading={loadingMovies}
			onpage={(p) => ondiscoverMovies(p, selectedGenreId)}
		/>
	{:else if error}
		<div class="alert alert-error">
			<p>{error}</p>
		</div>
	{:else}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No movies found. Try selecting a genre.</p>
		</div>
	{/if}
{:else if loadingTv}
	<div class="flex justify-center p-8">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else if tvShows.length > 0}
	<TmdbBrowseGrid {tvShows} {selectedTvShowId} {favoritedIds} {pinnedIds} {onselectTvShow} />
	<TmdbPagination
		page={tvPage}
		totalPages={tvTotalPages}
		loading={loadingTv}
		onpage={(p) => ondiscoverTv(p, selectedGenreId)}
	/>
{:else if error}
	<div class="alert alert-error">
		<p>{error}</p>
	</div>
{:else}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No TV shows found. Try selecting a genre.</p>
	</div>
{/if}
