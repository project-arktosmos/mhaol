<script lang="ts">
	import TmdbBrowseCard from './TmdbBrowseCard.svelte';
	import type { DisplayTMDBMovie } from 'addons/tmdb/types';

	let {
		movies,
		selectedMovieId = null,
		onselectMovie
	}: {
		movies: DisplayTMDBMovie[];
		selectedMovieId?: number | null;
		onselectMovie?: (movie: DisplayTMDBMovie) => void;
	} = $props();
</script>

{#if movies.length > 0}
	<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
		{#each movies as movie (movie.id)}
			<TmdbBrowseCard {movie} selected={selectedMovieId === movie.id} onclick={onselectMovie ? () => onselectMovie(movie) : undefined} />
		{/each}
	</div>
{:else}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No movies yet. Add a Movies library and scan it.</p>
	</div>
{/if}
