<script lang="ts">
	import classNames from 'classnames';
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import { playerService } from 'ui-lib/services/player.service';
	import type {
		DisplayTMDBMovie,
		DisplayTMDBMovieDetails,
		DisplayTMDBImage
	} from 'addons/tmdb/types';
	import type { MediaItem } from 'ui-lib/types/media-card.type';
	import type { LibraryItemRelated } from 'ui-lib/types/library-item-related.type';

	interface Props {
		movie: DisplayTMDBMovie;
		movieDetails: DisplayTMDBMovieDetails | null;
		libraryItem: MediaItem | null;
		relatedData: LibraryItemRelated | null;
		loading: boolean;
		fetching: boolean;
		fetched: boolean;
		fetchSteps: {
			terms: boolean;
			search: boolean;
			searching: boolean;
			eval: boolean;
			done: boolean;
		} | null;
		downloadStatus: { state: string; progress: number } | null;
		fetchedTorrent: { name: string; quality: string; languages: string } | null;
		imagesVisible: boolean;
		imageOverrides: Record<string, string> | null;
		onfetch: () => void;
		ondownload: () => void;
		onstream?: () => void;
		onp2pstream: () => void;
		onshowsearch: () => void;
		onback: () => void;
		ontoggleimages: () => void;
		onsetimageoverride: (filePath: string, role: 'poster' | 'backdrop') => void;
	}

	let {
		movie,
		movieDetails,
		libraryItem,
		relatedData,
		loading,
		fetching,
		fetched,
		fetchSteps,
		downloadStatus,
		fetchedTorrent,
		imagesVisible,
		imageOverrides,
		onfetch,
		ondownload,
		onstream,
		onp2pstream,
		onshowsearch,
		onback,
		ontoggleimages,
		onsetimageoverride
	}: Props = $props();

	let title = $derived(movie.title);
	let year = $derived(movie.releaseYear ?? '');
	let posterUrl = $derived(movie.posterUrl);
	let backdropUrl = $derived(movie.backdropUrl);
	let voteAverage = $derived(movie.voteAverage ?? 0);
	let voteCount = $derived(movie.voteCount ?? 0);
	let overview = $derived(movie.overview ?? '');
	let genres = $derived(movie.genres ?? []);
	let tagline = $derived(movieDetails?.tagline ?? null);
	let runtime = $derived(movieDetails?.runtime ?? null);
	let director = $derived(movieDetails?.director ?? null);
	let cast = $derived(movieDetails?.cast ?? []);
	let images: DisplayTMDBImage[] = $derived(movieDetails?.images ?? []);

	let dlState = $derived(downloadStatus?.state ?? null);
	let isDownloading = $derived(
		dlState === 'downloading' ||
			dlState === 'initializing' ||
			dlState === 'paused' ||
			dlState === 'checking'
	);
	let isDownloaded = $derived(dlState === 'completed' || dlState === 'seeding');
	let downloadButtonDisabled = $derived(!fetched || isDownloading || isDownloaded);
	let dlProgress = $derived(downloadStatus?.progress ?? 0);
	let dlPercent = $derived(Math.round(dlProgress * 100));
	let streamingTorrent = $state(false);
	let streamingP2p = $state(false);

	const playerState = playerService.state;
	const playerDisplayMode = playerService.displayMode;
</script>

<DetailPageLayout>
	<button class="btn self-start btn-ghost btn-sm" onclick={onback}>
		<svg
			xmlns="http://www.w3.org/2000/svg"
			class="h-4 w-4"
			fill="none"
			viewBox="0 0 24 24"
			stroke="currentColor"
			stroke-width="2"
		>
			<path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
		</svg>
		Back
	</button>

	{#if posterUrl}
		<img
			src={posterUrl}
			alt="{title} poster"
			class="w-full rounded-lg object-cover"
			loading="lazy"
		/>
	{/if}

	{#if images.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">All Images</h3>
			{#if !imagesVisible}
				<button class="btn w-full btn-outline btn-sm" onclick={ontoggleimages}>
					Show Images ({images.length})
				</button>
			{:else}
				<div class="grid grid-cols-3 gap-1">
					{#each images as image}
						<div class="group relative">
							<a href={image.fullUrl} target="_blank" rel="noopener noreferrer">
								<img
									src={image.thumbnailUrl}
									alt="{title} image"
									class={classNames(
										'w-full rounded object-cover transition-opacity hover:opacity-80',
										{
											'aspect-video': image.width > image.height,
											'aspect-[2/3]': image.width <= image.height,
											'ring-2 ring-success':
												imageOverrides?.poster === image.filePath ||
												imageOverrides?.backdrop === image.filePath
										}
									)}
									loading="lazy"
								/>
							</a>
							<div
								class="pointer-events-none absolute inset-0 flex items-end justify-center gap-1 rounded p-1 opacity-0 transition-opacity group-hover:pointer-events-auto group-hover:bg-black/40 group-hover:opacity-100"
							>
								<button
									class={classNames('btn btn-xs', {
										'btn-success': imageOverrides?.poster === image.filePath,
										'text-white btn-ghost': imageOverrides?.poster !== image.filePath
									})}
									onclick={(e: MouseEvent) => {
										e.preventDefault();
										e.stopPropagation();
										onsetimageoverride(image.filePath, 'poster');
									}}
								>
									Poster
								</button>
								<button
									class={classNames('btn btn-xs', {
										'btn-success': imageOverrides?.backdrop === image.filePath,
										'text-white btn-ghost': imageOverrides?.backdrop !== image.filePath
									})}
									onclick={(e: MouseEvent) => {
										e.preventDefault();
										e.stopPropagation();
										onsetimageoverride(image.filePath, 'backdrop');
									}}
								>
									Backdrop
								</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	{#if libraryItem}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Library Item</h3>
			<table class="table table-xs">
				<tbody>
					<tr
						><td class="font-medium opacity-60">ID</td><td class="break-all">{libraryItem.id}</td
						></tr
					>
					<tr
						><td class="font-medium opacity-60">Name</td><td class="break-all"
							>{libraryItem.name}</td
						></tr
					>
				</tbody>
			</table>
		</div>
	{/if}

	{#snippet cellA()}
		<h1 class="text-xl font-bold">{title}</h1>

		{#if tagline}
			<p class="text-sm italic opacity-60">{tagline}</p>
		{/if}

		<div class="flex flex-wrap items-center gap-2 text-sm">
			<span class="font-medium">{year}</span>
			{#if voteAverage > 0}
				<span class="flex items-center gap-1">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						viewBox="0 0 24 24"
						fill="currentColor"
						class="h-4 w-4 text-yellow-500"
					>
						<path
							fill-rule="evenodd"
							d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z"
							clip-rule="evenodd"
						/>
					</svg>
					<span class="font-semibold">{voteAverage.toFixed(1)}</span>
					<span class="text-xs opacity-50">({voteCount})</span>
				</span>
			{/if}
			<span class="badge badge-sm badge-primary">Movie</span>
		</div>

		{#if runtime}
			<p class="text-sm opacity-60">{runtime}</p>
		{/if}

		{#if genres.length > 0}
			<div class="flex flex-wrap gap-1">
				{#each genres as genre}
					<span class="badge badge-outline badge-sm">{genre}</span>
				{/each}
			</div>
		{/if}

		{#if backdropUrl}
			<img
				src={backdropUrl}
				alt="{title} backdrop"
				class="aspect-video w-full rounded-lg object-cover"
				loading="lazy"
			/>
		{/if}

		{#if overview}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Overview</h3>
				<p class="text-sm leading-relaxed opacity-80">{overview}</p>
			</div>
		{/if}

		{#if loading}
			<div class="flex justify-center py-4">
				<span class="loading loading-sm loading-spinner"></span>
			</div>
		{/if}

		{#if director}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Director</h3>
				<p class="text-sm">{director}</p>
			</div>
		{/if}

		{#if cast.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Cast</h3>
				<div class="flex flex-col gap-1">
					{#each cast.slice(0, 8) as member}
						<div class="flex items-center gap-2">
							{#if member.profileUrl}
								<img
									src={member.profileUrl}
									alt={member.name}
									class="h-8 w-8 rounded-full object-cover"
									loading="lazy"
								/>
							{:else}
								<div
									class="flex h-8 w-8 items-center justify-center rounded-full bg-base-300 text-xs opacity-40"
								>
									?
								</div>
							{/if}
							<div class="min-w-0">
								<p class="truncate text-sm font-medium">{member.name}</p>
								<p class="truncate text-xs opacity-50">{member.character}</p>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	{/snippet}

	{#snippet cellB()}
		{#if $playerState.currentFile && $playerDisplayMode === 'inline'}
			<div class="overflow-hidden rounded-lg bg-black">
				<div class="flex items-center justify-between px-2 py-1">
					<p
						class="min-w-0 truncate text-xs font-semibold text-white"
						title={$playerState.currentFile.name}
					>
						{$playerState.currentFile.name}
					</p>
					<div class="flex shrink-0 items-center gap-1">
						<button
							class="btn btn-square text-white btn-ghost btn-xs"
							onclick={() => playerService.setDisplayMode('fullscreen')}
							aria-label="Fullscreen player"
							title="Fullscreen player"
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								class="h-3.5 w-3.5"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
								stroke-width="2"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M4 8V4h4M20 8V4h-4M4 16v4h4M20 16v4h-4"
								/>
							</svg>
						</button>
						<button
							class="btn btn-square text-white btn-ghost btn-xs"
							onclick={() => playerService.stop()}
							aria-label="Close player"
						>
							&times;
						</button>
					</div>
				</div>
				<PlayerVideo
					file={$playerState.currentFile}
					connectionState={$playerState.connectionState}
					positionSecs={$playerState.positionSecs}
					durationSecs={$playerState.durationSecs}
					buffering={$playerState.buffering}
					fullscreen={false}
				/>
			</div>
		{/if}

		<div class="grid grid-cols-2 gap-2">
			<button
				class="btn col-span-2 btn-sm {fetched ? 'btn-ghost' : 'btn-info'}"
				onclick={onfetch}
				disabled={fetching}
			>
				{#if fetching}
					<span class="loading loading-xs loading-spinner"></span>
				{:else}
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
							stroke-width="2"
							d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
						/>
					</svg>
				{/if}
				Smart Search
			</button>
			{#if fetchSteps}
				<button
					class="col-span-2 cursor-pointer rounded-lg bg-base-200 p-2 transition-colors hover:bg-base-300"
					onclick={onshowsearch}
				>
					<ul class="steps steps-horizontal w-full text-xs">
						<li class={classNames('step', { 'step-success': fetchSteps.terms })}>Terms</li>
						<li class={classNames('step', { 'step-success': fetchSteps.search })}>
							{fetchSteps.searching ? 'Searching...' : 'Search'}
						</li>
						<li class={classNames('step', { 'step-success': fetchSteps.eval })}>Analysis</li>
						<li class={classNames('step', { 'step-success': fetchSteps.done })}>
							{fetchSteps.done ? 'Done' : 'Candidate'}
						</li>
					</ul>
				</button>
			{/if}
			{#if fetchedTorrent}
				<div class="col-span-2 flex items-center gap-2">
					<p class="min-w-0 flex-1 truncate text-xs opacity-60" title={fetchedTorrent.name}>
						{fetchedTorrent.name}
					</p>
					{#if fetchedTorrent.quality}
						<span class="badge badge-xs badge-info">{fetchedTorrent.quality}</span>
					{/if}
					{#if fetchedTorrent.languages}
						<span class="badge badge-ghost badge-xs">{fetchedTorrent.languages}</span>
					{/if}
				</div>
			{/if}
			<button
				class={classNames('btn btn-sm', {
					'btn-ghost': isDownloaded,
					'btn-success': !isDownloaded
				})}
				onclick={ondownload}
				disabled={downloadButtonDisabled}
			>
				{#if isDownloading}
					<span class="loading loading-xs loading-spinner"></span> Downloading
				{:else if isDownloaded}
					Downloaded
				{:else}
					Download
				{/if}
			</button>
			<button
				class="btn btn-sm btn-primary"
				onclick={() => {
					streamingTorrent = true;
					onstream?.();
				}}
				disabled={downloadButtonDisabled || streamingTorrent}
			>
				{#if streamingTorrent}
					<span class="loading loading-xs loading-spinner"></span>
				{/if}
				Stream Torrent
			</button>
			<button
				class="btn btn-sm btn-secondary"
				onclick={() => {
					streamingP2p = true;
					onp2pstream();
				}}
				disabled={!downloadButtonDisabled || streamingP2p}
			>
				{#if streamingP2p}
					<span class="loading loading-xs loading-spinner"></span>
				{/if}
				P2P Stream
			</button>
			{#if isDownloading || isDownloaded}
				<div class="col-span-2 flex items-center gap-2">
					<progress
						class={classNames('progress flex-1', {
							'progress-info': isDownloading,
							'progress-success': isDownloaded
						})}
						value={dlPercent}
						max="100"
					></progress>
					<span class="text-xs font-medium opacity-60">{dlPercent}%</span>
				</div>
			{/if}
		</div>
	{/snippet}
</DetailPageLayout>
