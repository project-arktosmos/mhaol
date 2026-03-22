<script lang="ts">
	import TmdbBrowseCard from './TmdbBrowseCard.svelte';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';
	import type { TorrentState } from 'frontend/types/torrent.type';

	interface DownloadStatus {
		state: TorrentState;
		progress: number;
	}

	let {
		movies = [],
		tvShows = [],
		selectedMovieId = null,
		selectedTvShowId = null,
		fetchedIds,
		downloadStatuses,
		onselectMovie,
		onselectTvShow
	}: {
		movies?: DisplayTMDBMovie[];
		tvShows?: DisplayTMDBTvShow[];
		selectedMovieId?: number | null;
		selectedTvShowId?: number | null;
		fetchedIds?: Set<number>;
		downloadStatuses?: Map<number, DownloadStatus>;
		onselectMovie?: (movie: DisplayTMDBMovie) => void;
		onselectTvShow?: (tvShow: DisplayTMDBTvShow) => void;
	} = $props();
</script>

<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
	{#each movies as movie (movie.id)}
		{@const dl = downloadStatuses?.get(movie.id)}
		<TmdbBrowseCard
			{movie}
			selected={selectedMovieId === movie.id}
			fetched={fetchedIds?.has(movie.id) ?? false}
			downloadState={dl?.state ?? null}
			downloadProgress={dl?.progress ?? null}
			onclick={onselectMovie ? () => onselectMovie(movie) : undefined}
		/>
	{/each}
	{#each tvShows as tvShow (tvShow.id)}
		<TmdbBrowseCard
			{tvShow}
			selected={selectedTvShowId === tvShow.id}
			onclick={onselectTvShow ? () => onselectTvShow(tvShow) : undefined}
		/>
	{/each}
</div>
