<script lang="ts">
	import { getContext } from 'svelte';
	import classNames from 'classnames';
	import CatalogCard from './CatalogCard.svelte';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';
	import type { CatalogCardData } from '$types/catalog.type';
	import type { TorrentState } from '$types/torrent.type';

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
		favoritedIds,
		pinnedIds,
		downloadStatuses,
		fetchCacheSummaries,
		dimmedIds,
		smartSearchingId = null,
		matchingTvShowId = null,
		onselectMovie,
		onselectTvShow,
		onsmartSearch
	}: {
		movies?: DisplayTMDBMovie[];
		tvShows?: DisplayTMDBTvShow[];
		selectedMovieId?: number | null;
		selectedTvShowId?: number | null;
		fetchedIds?: Set<number>;
		favoritedIds?: Set<number>;
		pinnedIds?: Set<number>;
		downloadStatuses?: Map<number, DownloadStatus>;
		fetchCacheSummaries?: Map<number, string>;
		dimmedIds?: Set<number>;
		smartSearchingId?: number | null;
		matchingTvShowId?: number | null;
		onselectMovie?: (movie: DisplayTMDBMovie) => void;
		onselectTvShow?: (tvShow: DisplayTMDBTvShow) => void;
		onsmartSearch?: (movie: DisplayTMDBMovie) => void;
	} = $props();

	let uniqueMovies = $derived(
		movies.filter((m, i, arr) => arr.findIndex((x) => x.id === m.id) === i)
	);
	let uniqueTvShows = $derived(
		tvShows.filter((s, i, arr) => arr.findIndex((x) => x.id === s.id) === i)
	);

	const browseViewMode = getContext<
		{ readonly value: 'poster' | 'backdrop' | 'table' } | undefined
	>('browseViewMode');
	let useBackdrop = $derived(browseViewMode?.value === 'backdrop');
	let useTable = $derived(browseViewMode?.value === 'table');

	function movieToCard(movie: DisplayTMDBMovie): CatalogCardData {
		const dl = downloadStatuses?.get(movie.id);
		return {
			kind: 'movie',
			id: String(movie.id),
			title: movie.title,
			subtitle: null,
			imageUrl: useBackdrop ? (movie.backdropUrl ?? movie.posterUrl) : movie.posterUrl,
			aspectRatio: useBackdrop ? 'landscape' : 'poster',
			badges: movie.genres.slice(0, 2).map((g) => ({ label: g, variant: 'ghost' })),
			rating: movie.voteAverage,
			year: movie.releaseYear || null,
			favorited: favoritedIds?.has(movie.id),
			pinned: pinnedIds?.has(movie.id),
			fetched: fetchedIds?.has(movie.id),
			selected: selectedMovieId === movie.id,
			dimmed: dimmedIds?.has(movie.id),
			loading: smartSearchingId === movie.id,
			torrentProgress: dl?.progress,
			torrentState: dl?.state,
			fetchCacheSummary: fetchCacheSummaries?.get(movie.id)
		};
	}

	function tvShowToCard(tvShow: DisplayTMDBTvShow): CatalogCardData {
		return {
			kind: 'tv_show',
			id: String(tvShow.id),
			title: tvShow.name,
			subtitle: null,
			imageUrl: useBackdrop ? (tvShow.backdropUrl ?? tvShow.posterUrl) : tvShow.posterUrl,
			aspectRatio: useBackdrop ? 'landscape' : 'poster',
			badges: [
				...tvShow.genres.slice(0, 2).map((g) => ({ label: g, variant: 'ghost' })),
				{ label: 'TV', variant: 'info' }
			],
			rating: tvShow.voteAverage,
			year: tvShow.firstAirYear || null,
			favorited: favoritedIds?.has(tvShow.id),
			pinned: pinnedIds?.has(tvShow.id),
			selected: selectedTvShowId === tvShow.id,
			loading: matchingTvShowId === tvShow.id
		};
	}
</script>

{#if useTable}
	<div class="overflow-x-auto">
		<table class="table table-sm">
			<thead>
				<tr>
					<th></th>
					<th>Title</th>
					<th>Year</th>
					<th>Rating</th>
					<th>Status</th>
					{#if onsmartSearch}
						<th>Smart Search</th>
					{/if}
				</tr>
			</thead>
			<tbody>
				{#each uniqueMovies as movie (movie.id)}
					{@const dl = downloadStatuses?.get(movie.id)}
					{@const fetched = fetchedIds?.has(movie.id) ?? false}
					<tr
						class={classNames('hover:bg-base-200', {
							'cursor-pointer': !!onselectMovie,
							'bg-primary/10': selectedMovieId === movie.id,
							'opacity-50': dimmedIds?.has(movie.id)
						})}
						onclick={onselectMovie ? () => onselectMovie(movie) : undefined}
					>
						<td class="w-12">
							{#if movie.posterUrl}
								<img
									src={movie.posterUrl}
									alt={movie.title}
									class="h-12 w-8 rounded object-cover"
									loading="lazy"
								/>
							{:else}
								<div
									class="flex h-12 w-8 items-center justify-center rounded bg-base-300 text-xs text-base-content/20"
								>
									?
								</div>
							{/if}
						</td>
						<td class="font-medium">{movie.title}</td>
						<td class="text-base-content/60">{movie.releaseYear}</td>
						<td>
							{#if movie.voteAverage > 0}
								<span class="text-sm">{movie.voteAverage.toFixed(1)}</span>
							{/if}
						</td>
						<td>
							<div class="flex gap-1">
								{#if favoritedIds?.has(movie.id)}
									<span class="text-xs text-red-500">♥</span>
								{/if}
								{#if pinnedIds?.has(movie.id)}
									<span class="text-xs text-blue-400">★</span>
								{/if}
								{#if dl}
									{@const label =
										dl.state === 'downloading' ? `${Math.round(dl.progress * 100)}%` : dl.state}
									<span
										class={classNames('badge badge-xs', {
											'badge-primary': dl.state === 'downloading',
											'badge-success': dl.state === 'seeding',
											'badge-warning': dl.state === 'paused',
											'badge-error': dl.state === 'error'
										})}>{label}</span
									>
								{/if}
								{#if fetched}
									<span class="badge badge-xs badge-success">fetched</span>
								{/if}
							</div>
						</td>
						{#if onsmartSearch}
							<td>
								{#if fetchCacheSummaries?.get(movie.id)}
									<span
										class="block max-w-xs truncate text-xs opacity-70"
										title={fetchCacheSummaries.get(movie.id)}
									>
										{fetchCacheSummaries.get(movie.id)}
									</span>
								{:else if smartSearchingId === movie.id}
									<span class="loading loading-xs loading-spinner"></span>
								{:else}
									<button
										class="btn btn-outline btn-xs"
										onclick={(e) => {
											e.stopPropagation();
											onsmartSearch(movie);
										}}
									>
										Search
									</button>
								{/if}
							</td>
						{/if}
					</tr>
				{/each}
				{#each uniqueTvShows as tvShow (tvShow.id)}
					<tr
						class={classNames('hover:bg-base-200', {
							'cursor-pointer': !!onselectTvShow,
							'bg-primary/10': selectedTvShowId === tvShow.id
						})}
						onclick={onselectTvShow ? () => onselectTvShow(tvShow) : undefined}
					>
						<td class="w-12">
							{#if tvShow.posterUrl}
								<img
									src={tvShow.posterUrl}
									alt={tvShow.name}
									class="h-12 w-8 rounded object-cover"
									loading="lazy"
								/>
							{:else}
								<div
									class="flex h-12 w-8 items-center justify-center rounded bg-base-300 text-xs text-base-content/20"
								>
									?
								</div>
							{/if}
						</td>
						<td class="font-medium">{tvShow.name}</td>
						<td class="text-base-content/60">{tvShow.firstAirYear}</td>
						<td>
							{#if tvShow.voteAverage > 0}
								<span class="text-sm">{tvShow.voteAverage.toFixed(1)}</span>
							{/if}
						</td>
						<td>
							<div class="flex gap-1">
								{#if favoritedIds?.has(tvShow.id)}
									<span class="text-xs text-red-500">♥</span>
								{/if}
								{#if pinnedIds?.has(tvShow.id)}
									<span class="text-xs text-blue-400">★</span>
								{/if}
								<span class="badge badge-xs badge-info">TV</span>
							</div>
						</td>
						{#if onsmartSearch}
							<td></td>
						{/if}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{:else}
	<div
		class={classNames(
			'grid gap-4',
			useBackdrop
				? 'grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4'
				: 'grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6'
		)}
	>
		{#each uniqueMovies as movie (movie.id)}
			<CatalogCard
				card={movieToCard(movie)}
				onclick={onselectMovie ? () => onselectMovie(movie) : undefined}
			/>
		{/each}
		{#each uniqueTvShows as tvShow (tvShow.id)}
			<CatalogCard
				card={tvShowToCard(tvShow)}
				onclick={onselectTvShow ? () => onselectTvShow(tvShow) : undefined}
			/>
		{/each}
	</div>
{/if}
