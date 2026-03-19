<script lang="ts">
	import classNames from 'classnames';
	import TmdbBrowseCard from './TmdbBrowseCard.svelte';
	import TmdbPagination from './TmdbPagination.svelte';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';
	import type { TmdbGenre } from '$types/tmdb-browse.type';

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
		ondiscoverMovies: (page: number, genreId: number | null) => void;
		ondiscoverTv: (page: number, genreId: number | null) => void;
	} = $props();

	let subTab = $state<'movies' | 'tv'>('movies');

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
	<button
		class={classNames('btn btn-xs', {
			'btn-secondary': subTab === 'movies',
			'btn-ghost': subTab !== 'movies'
		})}
		onclick={() => (subTab = 'movies')}
	>
		Movies
	</button>
	<button
		class={classNames('btn btn-xs', {
			'btn-secondary': subTab === 'tv',
			'btn-ghost': subTab !== 'tv'
		})}
		onclick={() => (subTab = 'tv')}
	>
		TV Shows
	</button>

	<select
		class="select-bordered select ml-auto select-xs"
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
		<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
			{#each movies as movie (movie.id)}
				<TmdbBrowseCard {movie} />
			{/each}
		</div>
		<TmdbPagination
			page={moviesPage}
			totalPages={moviesTotalPages}
			loading={loadingMovies}
			onpage={(p) => ondiscoverMovies(p, selectedGenreId)}
		/>
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
	<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
		{#each tvShows as tvShow (tvShow.id)}
			<TmdbBrowseCard {tvShow} />
		{/each}
	</div>
	<TmdbPagination
		page={tvPage}
		totalPages={tvTotalPages}
		loading={loadingTv}
		onpage={(p) => ondiscoverTv(p, selectedGenreId)}
	/>
{:else}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No TV shows found. Try selecting a genre.</p>
	</div>
{/if}
