<script lang="ts">
	import { getContext } from 'svelte';
	import classNames from 'classnames';
	import TmdbBrowseCard from './TmdbBrowseCard.svelte';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';
	import type { TorrentState } from 'ui-lib/types/torrent.type';

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

	// Deduplicate by id to avoid Svelte each_key_duplicate errors (TMDB API can return duplicates)
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
					<th>Genres</th>
					<th></th>
					<th>Status</th>
					<th>Smart Search</th>
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
									class="flex h-12 w-8 items-center justify-center rounded bg-base-300 text-base-content/20"
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										class="h-4 w-4"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
									>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="1"
											d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"
										/>
									</svg>
								</div>
							{/if}
						</td>
						<td class="font-medium">{movie.title}</td>
						<td class="text-base-content/60">{movie.releaseYear}</td>
						<td>
							{#if movie.voteAverage > 0}
								<span class="flex items-center gap-0.5">
									<svg
										xmlns="http://www.w3.org/2000/svg"
										viewBox="0 0 24 24"
										fill="currentColor"
										class="h-3 w-3 text-yellow-500"
									>
										<path
											fill-rule="evenodd"
											d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
											clip-rule="evenodd"
										/>
									</svg>
									{movie.voteAverage.toFixed(1)}
								</span>
							{/if}
						</td>
						<td>
							{#if movie.genres.length > 0}
								<div class="flex flex-wrap gap-1">
									{#each movie.genres.slice(0, 3) as genre}
										<span class="badge badge-outline badge-xs">{genre}</span>
									{/each}
								</div>
							{/if}
						</td>
						<td>
							<div class="flex gap-1">
								{#if favoritedIds?.has(movie.id)}
									<svg
										xmlns="http://www.w3.org/2000/svg"
										viewBox="0 0 24 24"
										fill="currentColor"
										class="h-3.5 w-3.5 text-red-500"
									>
										<path
											d="M11.645 20.91l-.007-.003-.022-.012a15.247 15.247 0 01-.383-.218 25.18 25.18 0 01-4.244-3.17C4.688 15.36 2.25 12.174 2.25 8.25 2.25 5.322 4.714 3 7.688 3A5.5 5.5 0 0112 5.052 5.5 5.5 0 0116.313 3c2.973 0 5.437 2.322 5.437 5.25 0 3.925-2.438 7.111-4.739 9.256a25.175 25.175 0 01-4.244 3.17 15.247 15.247 0 01-.383.219l-.022.012-.007.004-.003.001a.752.752 0 01-.704 0l-.003-.001z"
										/>
									</svg>
								{/if}
								{#if pinnedIds?.has(movie.id)}
									<svg
										xmlns="http://www.w3.org/2000/svg"
										viewBox="0 0 24 24"
										fill="currentColor"
										class="h-3.5 w-3.5 text-blue-400"
									>
										<path
											fill-rule="evenodd"
											d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
											clip-rule="evenodd"
										/>
									</svg>
								{/if}
							</div>
						</td>
						<td>
							{#if dl}
								{@const label =
									dl.state === 'downloading' && dl.progress !== null
										? `${Math.round(dl.progress * 100)}%`
										: dl.state === 'seeding'
											? 'Seeding'
											: dl.state === 'paused'
												? 'Paused'
												: dl.state === 'error'
													? 'Error'
													: dl.state}
								{@const badgeClass =
									dl.state === 'downloading'
										? 'badge-primary'
										: dl.state === 'seeding'
											? 'badge-success'
											: dl.state === 'paused'
												? 'badge-warning'
												: dl.state === 'error'
													? 'badge-error'
													: 'badge-info'}
								<span class={classNames('badge badge-xs', badgeClass)}>{label}</span>
							{/if}
							{#if fetched}
								<span class="badge badge-xs badge-success">smart searched</span>
							{/if}
						</td>
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
							{:else if onsmartSearch}
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
									class="flex h-12 w-8 items-center justify-center rounded bg-base-300 text-base-content/20"
								>
									<svg
										xmlns="http://www.w3.org/2000/svg"
										class="h-4 w-4"
										fill="none"
										viewBox="0 0 24 24"
										stroke="currentColor"
									>
										<path
											stroke-linecap="round"
											stroke-linejoin="round"
											stroke-width="1"
											d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"
										/>
									</svg>
								</div>
							{/if}
						</td>
						<td class="font-medium">{tvShow.name}</td>
						<td class="text-base-content/60">{tvShow.firstAirYear}</td>
						<td>
							{#if tvShow.voteAverage > 0}
								<span class="flex items-center gap-0.5">
									<svg
										xmlns="http://www.w3.org/2000/svg"
										viewBox="0 0 24 24"
										fill="currentColor"
										class="h-3 w-3 text-yellow-500"
									>
										<path
											fill-rule="evenodd"
											d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
											clip-rule="evenodd"
										/>
									</svg>
									{tvShow.voteAverage.toFixed(1)}
								</span>
							{/if}
						</td>
						<td>
							{#if tvShow.genres.length > 0}
								<div class="flex flex-wrap gap-1">
									{#each tvShow.genres.slice(0, 3) as genre}
										<span class="badge badge-outline badge-xs">{genre}</span>
									{/each}
								</div>
							{/if}
						</td>
						<td>
							<div class="flex gap-1">
								{#if favoritedIds?.has(tvShow.id)}
									<svg
										xmlns="http://www.w3.org/2000/svg"
										viewBox="0 0 24 24"
										fill="currentColor"
										class="h-3.5 w-3.5 text-red-500"
									>
										<path
											d="M11.645 20.91l-.007-.003-.022-.012a15.247 15.247 0 01-.383-.218 25.18 25.18 0 01-4.244-3.17C4.688 15.36 2.25 12.174 2.25 8.25 2.25 5.322 4.714 3 7.688 3A5.5 5.5 0 0112 5.052 5.5 5.5 0 0116.313 3c2.973 0 5.437 2.322 5.437 5.25 0 3.925-2.438 7.111-4.739 9.256a25.175 25.175 0 01-4.244 3.17 15.247 15.247 0 01-.383.219l-.022.012-.007.004-.003.001a.752.752 0 01-.704 0l-.003-.001z"
										/>
									</svg>
								{/if}
								{#if pinnedIds?.has(tvShow.id)}
									<svg
										xmlns="http://www.w3.org/2000/svg"
										viewBox="0 0 24 24"
										fill="currentColor"
										class="h-3.5 w-3.5 text-blue-400"
									>
										<path
											fill-rule="evenodd"
											d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
											clip-rule="evenodd"
										/>
									</svg>
								{/if}
							</div>
						</td>
						<td>
							<span class="badge badge-xs badge-info">TV</span>
						</td>
						<td></td>
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
			{@const dl = downloadStatuses?.get(movie.id)}
			<TmdbBrowseCard
				{movie}
				selected={selectedMovieId === movie.id}
				fetched={fetchedIds?.has(movie.id) ?? false}
				dimmed={dimmedIds?.has(movie.id) ?? false}
				favorited={favoritedIds?.has(movie.id) ?? false}
				pinned={pinnedIds?.has(movie.id) ?? false}
				downloadState={dl?.state ?? null}
				downloadProgress={dl?.progress ?? null}
				onclick={onselectMovie ? () => onselectMovie(movie) : undefined}
			/>
		{/each}
		{#each uniqueTvShows as tvShow (tvShow.id)}
			<TmdbBrowseCard
				{tvShow}
				selected={selectedTvShowId === tvShow.id}
				matching={matchingTvShowId === tvShow.id}
				favorited={favoritedIds?.has(tvShow.id) ?? false}
				pinned={pinnedIds?.has(tvShow.id) ?? false}
				onclick={onselectTvShow ? () => onselectTvShow(tvShow) : undefined}
			/>
		{/each}
	</div>
{/if}
