<script lang="ts">
	import { getContext } from 'svelte';
	import classNames from 'classnames';
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

	const browseViewMode = getContext<{ readonly value: 'poster' | 'backdrop' } | undefined>('browseViewMode');
	let useBackdrop = $derived(browseViewMode?.value === 'backdrop');
</script>

<div class={classNames('grid gap-4', useBackdrop ? 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4' : 'grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6')}>
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
