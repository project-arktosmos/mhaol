<script lang="ts">
	import TmdbBrowseGrid from './TmdbBrowseGrid.svelte';
	import type { DisplayTMDBMovie } from 'addons/tmdb/types';
	import type { TorrentState } from 'ui-lib/types/torrent.type';

	let {
		movies,
		selectedMovieId = null,
		fetchedIds = new Set<number>(),
		downloadStatuses,
		onselectMovie
	}: {
		movies: DisplayTMDBMovie[];
		selectedMovieId?: number | null;
		fetchedIds?: Set<number>;
		downloadStatuses?: Map<number, { state: TorrentState; progress: number }>;
		onselectMovie?: (movie: DisplayTMDBMovie) => void;
	} = $props();
</script>

{#if movies.length > 0}
	<TmdbBrowseGrid {movies} {selectedMovieId} {fetchedIds} {downloadStatuses} {onselectMovie} />
{:else}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">No movies yet. Add a Movies library and scan it.</p>
	</div>
{/if}
