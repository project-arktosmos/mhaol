<script lang="ts">
	import { getContext } from 'svelte';
	import classNames from 'classnames';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';
	import type { TorrentState } from 'ui-lib/types/torrent.type';

	let {
		movie = null,
		tvShow = null,
		selected = false,
		fetched = false,
		dimmed = false,
		matching = false,
		downloadState = null,
		downloadProgress = null,
		onclick
	}: {
		movie?: DisplayTMDBMovie | null;
		tvShow?: DisplayTMDBTvShow | null;
		selected?: boolean;
		fetched?: boolean;
		dimmed?: boolean;
		matching?: boolean;
		downloadState?: TorrentState | null;
		downloadProgress?: number | null;
		onclick?: () => void;
	} = $props();

	const browseViewMode = getContext<
		{ readonly value: 'poster' | 'backdrop' | 'table' } | undefined
	>('browseViewMode');
	let useBackdrop = $derived(browseViewMode?.value === 'backdrop');

	let title = $derived(movie?.title ?? tvShow?.name ?? '');
	let year = $derived(movie?.releaseYear ?? tvShow?.firstAirYear ?? '');
	let posterUrl = $derived(movie?.posterUrl ?? tvShow?.posterUrl ?? null);
	let backdropUrl = $derived(movie?.backdropUrl ?? tvShow?.backdropUrl ?? null);
	let imageUrl = $derived(useBackdrop ? (backdropUrl ?? posterUrl) : posterUrl);
	let voteAverage = $derived(movie?.voteAverage ?? tvShow?.voteAverage ?? 0);
	let overview = $derived(movie?.overview ?? tvShow?.overview ?? '');
	let genres = $derived(movie?.genres ?? tvShow?.genres ?? []);
	let isMovie = $derived(movie !== null);

	let downloadBadgeClass = $derived.by(() => {
		switch (downloadState) {
			case 'downloading':
				return 'badge-primary';
			case 'seeding':
				return 'badge-success';
			case 'paused':
				return 'badge-warning';
			case 'error':
				return 'badge-error';
			case 'initializing':
			case 'checking':
				return 'badge-info';
			default:
				return '';
		}
	});

	let downloadLabel = $derived.by(() => {
		if (!downloadState) return '';
		if (downloadState === 'downloading' && downloadProgress !== null) {
			return `${Math.round(downloadProgress * 100)}%`;
		}
		if (downloadState === 'seeding') return 'Seeding';
		if (downloadState === 'paused') return 'Paused';
		if (downloadState === 'error') return 'Error';
		if (downloadState === 'initializing') return 'Init';
		if (downloadState === 'checking') return 'Checking';
		return downloadState;
	});
</script>

{#if isMovie}
	<div
		class={classNames('relative overflow-hidden rounded-lg bg-base-300', {
			'cursor-pointer transition-shadow hover:shadow-md': !!onclick,
			'ring-2 ring-primary': selected,
			'opacity-50': dimmed
		})}
		{onclick}
		role={onclick ? 'button' : undefined}
		tabindex={onclick ? 0 : undefined}
		onkeydown={onclick
			? (e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						onclick?.();
					}
				}
			: undefined}
	>
		{#if fetched || downloadState}
			<div class="absolute top-1.5 right-1.5 z-10 flex gap-1">
				{#if downloadState}
					<span class={classNames('badge gap-0.5 badge-xs', downloadBadgeClass)}>
						{downloadLabel}
					</span>
				{/if}
				{#if fetched}
					<span class="badge gap-0.5 badge-xs badge-success">
						<svg
							xmlns="http://www.w3.org/2000/svg"
							viewBox="0 0 20 20"
							fill="currentColor"
							class="h-2.5 w-2.5"
						>
							<path
								fill-rule="evenodd"
								d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z"
								clip-rule="evenodd"
							/>
						</svg>
						smart searched
					</span>
				{/if}
			</div>
		{/if}
		{#if downloadState === 'downloading' && downloadProgress !== null}
			<div class="absolute bottom-0 left-0 z-10 w-full">
				<progress
					class="progress h-1 w-full progress-primary"
					value={downloadProgress * 100}
					max="100"
				></progress>
			</div>
		{/if}
		{#if imageUrl}
			<img
				src={imageUrl}
				alt={title}
				class={classNames('block w-full', { 'aspect-video object-cover': useBackdrop })}
				loading="lazy"
			/>
		{:else}
			<div
				class={classNames(
					'flex w-full items-center justify-center text-base-content/20',
					useBackdrop ? 'aspect-video' : 'aspect-2/3'
				)}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="h-12 w-12"
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
		{#if useBackdrop}
			<div
				class="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 to-transparent px-2 pt-6 pb-2"
			>
				<p class="truncate text-sm font-semibold text-white drop-shadow">{title}</p>
			</div>
		{/if}
	</div>
{:else}
	<div
		class={classNames('group card-compact card relative border border-info/40 bg-base-200 shadow-sm', {
			'cursor-pointer transition-shadow hover:shadow-md': !!onclick,
			'ring-2 ring-primary': selected
		})}
		{onclick}
		role={onclick ? 'button' : undefined}
		tabindex={onclick ? 0 : undefined}
		onkeydown={onclick
			? (e) => {
					if (e.key === 'Enter' || e.key === ' ') {
						e.preventDefault();
						onclick?.();
					}
				}
			: undefined}
	>
		{#if matching}
			<div class="absolute inset-0 z-20 flex items-center justify-center rounded-lg bg-base-300/70">
				<span class="loading loading-md loading-spinner text-primary"></span>
			</div>
		{/if}
		<figure class="relative h-48 overflow-hidden bg-base-300">
			{#if imageUrl}
				<img src={imageUrl} alt={title} class="h-full w-full object-cover" loading="lazy" />
			{:else}
				<div class="flex h-full w-full items-center justify-center text-base-content/20">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-12 w-12"
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
		</figure>
		<div class="card-body gap-1">
			<h3 class="card-title truncate text-sm" {title}>{title}</h3>
			<div class="flex flex-wrap items-center gap-1 text-xs">
				<span class="opacity-60">{year}</span>
				{#if voteAverage > 0}
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
						<span class="font-semibold">{voteAverage.toFixed(1)}</span>
					</span>
				{/if}
				<span class="badge badge-xs badge-info">TV</span>
			</div>
			{#if genres.length > 0}
				<div class="flex flex-wrap gap-1">
					{#each genres.slice(0, 3) as genre}
						<span class="badge badge-outline badge-xs">{genre}</span>
					{/each}
				</div>
			{/if}
			{#if overview}
				<p class="line-clamp-2 text-xs opacity-60">{overview}</p>
			{/if}
		</div>
	</div>
{/if}
