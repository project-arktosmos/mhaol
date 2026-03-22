<script lang="ts">
	import classNames from 'classnames';
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
		loadingMovies = false,
		loadingTv = false,
		error = null,
		mediaType,
		selectedMovieId = null,
		selectedTvShowId = null,
		fetchedIds,
		downloadStatuses,
		onselectMovie,
		onselectTvShow,
		onloadMovies,
		onloadTv
	}: {
		movies: DisplayTMDBMovie[];
		tvShows: DisplayTMDBTvShow[];
		moviesPage: number;
		tvPage: number;
		moviesTotalPages: number;
		tvTotalPages: number;
		loadingMovies?: boolean;
		loadingTv?: boolean;
		error?: string | null;
		mediaType?: 'movies' | 'tv';
		selectedMovieId?: number | null;
		selectedTvShowId?: number | null;
		fetchedIds?: Set<number>;
		downloadStatuses?: Map<number, { state: TorrentState; progress: number }>;
		onselectMovie?: (movie: DisplayTMDBMovie) => void;
		onselectTvShow?: (tvShow: DisplayTMDBTvShow) => void;
		onloadMovies: (page: number) => void;
		onloadTv: (page: number) => void;
	} = $props();

	let subTabInternal = $state<'movies' | 'tv'>('movies');
	let subTab = $derived(mediaType ?? subTabInternal);
</script>

{#if !mediaType}
	<div class="mb-4 flex gap-2">
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
	</div>
{/if}

{#if subTab === 'movies'}
	{#if loadingMovies}
		<div class="flex justify-center p-8">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if movies.length > 0}
		<TmdbBrowseGrid {movies} {selectedMovieId} {fetchedIds} {downloadStatuses} {onselectMovie} />
		<TmdbPagination
			page={moviesPage}
			totalPages={moviesTotalPages}
			loading={loadingMovies}
			onpage={onloadMovies}
		/>
	{:else if error}
		<div class="alert alert-error">
			<p>{error}</p>
		</div>
	{:else}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No popular movies found.</p>
		</div>
	{/if}
{:else if loadingTv}
	<div class="flex justify-center p-8">
		<span class="loading loading-lg loading-spinner"></span>
	</div>
{:else if tvShows.length > 0}
	<TmdbBrowseGrid {tvShows} {selectedTvShowId} {onselectTvShow} />
	<TmdbPagination page={tvPage} totalPages={tvTotalPages} loading={loadingTv} onpage={onloadTv} />
{:else if error}
	<div class="alert alert-error">
		<p>{error}</p>
	</div>
{:else}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No popular TV shows found.</p>
	</div>
{/if}
