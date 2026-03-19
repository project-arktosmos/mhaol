<script lang="ts">
	import classNames from 'classnames';
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
		loadingMovies = false,
		loadingTv = false,
		mediaType,
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
		mediaType?: 'movies' | 'tv';
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
		<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
			{#each movies as movie (movie.id)}
				<TmdbBrowseCard {movie} onclick={onselectMovie ? () => onselectMovie(movie) : undefined} />
			{/each}
		</div>
		<TmdbPagination
			page={moviesPage}
			totalPages={moviesTotalPages}
			loading={loadingMovies}
			onpage={onloadMovies}
		/>
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
	<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
		{#each tvShows as tvShow (tvShow.id)}
			<TmdbBrowseCard {tvShow} onclick={onselectTvShow ? () => onselectTvShow(tvShow) : undefined} />
		{/each}
	</div>
	<TmdbPagination page={tvPage} totalPages={tvTotalPages} loading={loadingTv} onpage={onloadTv} />
{:else}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No popular TV shows found.</p>
	</div>
{/if}
