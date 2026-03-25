<script lang="ts">
	import classNames from 'classnames';
	import { smartSearchService } from 'ui-lib/services/smart-search.service';
	import type {
		DisplayTMDBMovie,
		DisplayTMDBTvShow,
		DisplayTMDBMovieDetails,
		DisplayTMDBTvShowDetails,
		DisplayTMDBSeasonDetails,
		DisplayTMDBImage
	} from 'addons/tmdb/types';
	import type { MediaItem } from 'ui-lib/types/media-card.type';
	import type { LibraryItemRelated } from 'ui-lib/types/library-item-related.type';
	import type { PlayableFile, PlayerConnectionState } from 'ui-lib/types/player.type';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	let expandedSeason = $state<number | null>(null);
	const smartSearchStore = smartSearchService.store;
	let tvMatchedSeasons = $derived.by(() => {
		const tvResults = $smartSearchStore.tvResults;
		if (!tvResults) return { hasComplete: false, seasons: new Map<number, Set<number>>() };
		const hasComplete = tvResults.complete.length > 0;
		const seasons = new Map<number, Set<number>>();
		for (const [snStr, data] of Object.entries(tvResults.seasons)) {
			const sn = Number(snStr);
			const eps = new Set<number>();
			if (data.seasonPacks.length > 0) eps.add(-1);
			for (const en of Object.keys(data.episodes).map(Number)) {
				if (data.episodes[en].length > 0) eps.add(en);
			}
			if (eps.size > 0) seasons.set(sn, eps);
		}
		return { hasComplete, seasons };
	});
	let {
		movie = null,
		tvShow = null,
		movieDetails = null,
		tvShowDetails = null,
		tvSeasonDetails = [],
		libraryItem = null,
		relatedData = null,
		loading = false,
		fetching = false,
		fetched = false,
		fetchSteps = null,
		playerFile = null,
		playerConnectionState = 'idle',
		playerPositionSecs = 0,
		playerDurationSecs = 0,
		playerBuffering = false,
		playerFullscreen = false,
		downloadStatus = null,
		onfetch,
		ondownload,
		onp2pstream,
		onfullscreen,
		onminimize,
		onstopplayer,
		onclose,
		onshowsearch
	}: {
		movie?: DisplayTMDBMovie | null;
		tvShow?: DisplayTMDBTvShow | null;
		movieDetails?: DisplayTMDBMovieDetails | null;
		tvShowDetails?: DisplayTMDBTvShowDetails | null;
		tvSeasonDetails?: DisplayTMDBSeasonDetails[];
		libraryItem?: MediaItem | null;
		relatedData?: LibraryItemRelated | null;
		loading?: boolean;
		fetching?: boolean;
		fetched?: boolean;
		fetchSteps?: {
			terms: boolean;
			search: boolean;
			searching: boolean;
			eval: boolean;
			done: boolean;
		} | null;
		playerFile?: PlayableFile | null;
		playerConnectionState?: PlayerConnectionState;
		playerPositionSecs?: number;
		playerDurationSecs?: number | null;
		playerBuffering?: boolean;
		playerFullscreen?: boolean;
		downloadStatus?: { state: string; progress: number } | null;
		onfetch?: () => void;
		ondownload?: () => void;
		onp2pstream?: () => void;
		onfullscreen?: () => void;
		onminimize?: () => void;
		onstopplayer?: () => void;
		onclose?: () => void;
		onshowsearch?: () => void;
	} = $props();

	let title = $derived(movie?.title ?? tvShow?.name ?? '');
	let year = $derived(movie?.releaseYear ?? tvShow?.firstAirYear ?? '');
	let posterUrl = $derived(movie?.posterUrl ?? tvShow?.posterUrl ?? null);
	let backdropUrl = $derived(movie?.backdropUrl ?? tvShow?.backdropUrl ?? null);
	let voteAverage = $derived(movie?.voteAverage ?? tvShow?.voteAverage ?? 0);
	let voteCount = $derived(movie?.voteCount ?? tvShow?.voteCount ?? 0);
	let overview = $derived(movie?.overview ?? tvShow?.overview ?? '');
	let genres = $derived(movie?.genres ?? tvShow?.genres ?? []);

	let tagline = $derived(movieDetails?.tagline ?? tvShowDetails?.tagline ?? null);
	let runtime = $derived(movieDetails?.runtime ?? null);
	let director = $derived(movieDetails?.director ?? null);
	let cast = $derived(movieDetails?.cast ?? tvShowDetails?.cast ?? []);
	let createdBy = $derived(tvShowDetails?.createdBy ?? []);
	let networks = $derived(tvShowDetails?.networks ?? []);
	let status = $derived(tvShowDetails?.status ?? null);
	let numberOfSeasons = $derived(tvShow?.numberOfSeasons ?? tvShowDetails?.numberOfSeasons ?? null);
	let numberOfEpisodes = $derived(
		tvShow?.numberOfEpisodes ?? tvShowDetails?.numberOfEpisodes ?? null
	);
	let lastAirYear = $derived(tvShow?.lastAirYear ?? tvShowDetails?.lastAirYear ?? null);
	let images: DisplayTMDBImage[] = $derived(movieDetails?.images ?? tvShowDetails?.images ?? []);
	let heroImageUrl = $derived(images.length > 0 ? images[0].thumbnailUrl : backdropUrl);

	function formatBytes(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
		return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
	}

	function formatProgress(p: number): string {
		return `${(p * 100).toFixed(1)}%`;
	}

	let dlState = $derived(downloadStatus?.state ?? null);
	let isDownloading = $derived(
		dlState === 'downloading' ||
			dlState === 'initializing' ||
			dlState === 'paused' ||
			dlState === 'checking'
	);
	let isDownloaded = $derived(dlState === 'completed' || dlState === 'seeding');
	let downloadButtonDisabled = $derived(!fetched || isDownloading || isDownloaded);
</script>

<div class="flex h-full flex-col overflow-y-auto">
	<div class="flex items-center justify-between p-3">
		<h2 class="min-w-0 truncate text-sm font-semibold" {title}>{title}</h2>
		{#if onclose}
			<button class="btn btn-square shrink-0 btn-ghost btn-xs" onclick={onclose} aria-label="Close">
				&times;
			</button>
		{/if}
	</div>

	{#if playerFile}
		<div
			class={classNames('bg-black', {
				'fixed inset-0 z-50 flex flex-col': playerFullscreen
			})}
		>
			<div
				class={classNames('flex items-center justify-between px-2 py-1', {
					'p-3': playerFullscreen
				})}
			>
				<p
					class={classNames('min-w-0 truncate font-semibold text-white', {
						'text-xs': !playerFullscreen,
						'text-sm': playerFullscreen
					})}
					title={playerFile.name}
				>
					{playerFile.name}
				</p>
				<div class="flex shrink-0 items-center gap-1">
					{#if playerFullscreen && onminimize}
						<button
							class="btn btn-square text-white btn-ghost btn-sm"
							onclick={onminimize}
							aria-label="Move to sidebar"
							title="Move to sidebar"
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								class="h-4 w-4"
								fill="none"
								viewBox="0 0 24 24"
								stroke="currentColor"
								stroke-width="2"
							>
								<path stroke-linecap="round" stroke-linejoin="round" d="M18 8L14 12L18 16" />
								<rect x="3" y="3" width="18" height="18" rx="2" />
								<line x1="14" y1="3" x2="14" y2="21" />
							</svg>
						</button>
					{:else if !playerFullscreen && onfullscreen}
						<button
							class="btn btn-square text-white btn-ghost btn-xs"
							onclick={onfullscreen}
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
					{/if}
					{#if onstopplayer}
						<button
							class={classNames('btn btn-square text-white btn-ghost', {
								'btn-sm': playerFullscreen,
								'btn-xs': !playerFullscreen
							})}
							onclick={onstopplayer}
							aria-label="Close player"
						>
							&times;
						</button>
					{/if}
				</div>
			</div>
			<div class={playerFullscreen ? 'min-h-0 flex-1' : ''}>
				<PlayerVideo
					file={playerFile}
					connectionState={playerConnectionState}
					positionSecs={playerPositionSecs}
					durationSecs={playerDurationSecs}
					buffering={playerBuffering}
					fullscreen={playerFullscreen}
				/>
			</div>
		</div>
	{:else if heroImageUrl}
		<div class="relative">
			<img src={heroImageUrl} alt={title} class="h-40 w-full object-cover" />
			<div class="absolute inset-0 bg-gradient-to-t from-base-200 to-transparent"></div>
		</div>
	{:else if posterUrl}
		<div class="flex justify-center bg-base-300 p-4">
			<img src={posterUrl} alt={title} class="h-48 rounded-lg object-cover" />
		</div>
	{/if}

	<div class="flex flex-col gap-3 p-3">
		{#if tagline}
			<p class="text-sm italic opacity-60">{tagline}</p>
		{/if}

		<div class="flex flex-wrap items-center gap-2 text-sm">
			<span class="font-medium">{year}{lastAirYear ? `\u2013${lastAirYear}` : ''}</span>
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
			{#if tvShow}
				<span class="badge badge-sm badge-info">TV</span>
			{:else}
				<span class="badge badge-sm badge-primary">Movie</span>
			{/if}
		</div>

		{#if runtime}
			<p class="text-xs opacity-60">{runtime}</p>
		{/if}

		{#if numberOfSeasons != null}
			<p class="text-xs opacity-60">
				{numberOfSeasons} season{numberOfSeasons !== 1 ? 's' : ''}{numberOfEpisodes != null
					? `, ${numberOfEpisodes} episodes`
					: ''}
				{#if status}&middot; {status}{/if}
			</p>
		{/if}

		{#if genres.length > 0}
			<div class="flex flex-wrap gap-1">
				{#each genres as genre}
					<span class="badge badge-outline badge-sm">{genre}</span>
				{/each}
			</div>
		{/if}

		<div class="grid grid-cols-2 gap-2">
			{#if onfetch}
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
			{/if}
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
			{#if ondownload}
				<button
					class={classNames('btn btn-sm', {
						'btn-ghost': isDownloaded,
						'btn-success': !isDownloaded
					})}
					onclick={ondownload}
					disabled={downloadButtonDisabled}
				>
					{#if isDownloading}
						<span class="loading loading-xs loading-spinner"></span>
						Downloading
					{:else if isDownloaded}
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
								d="M5 13l4 4L19 7"
							/>
						</svg>
						Downloaded
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
								d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
							/>
						</svg>
						Download
					{/if}
				</button>
			{/if}
			{#if onp2pstream}
				<button class="btn btn-sm btn-secondary" onclick={onp2pstream} disabled={!fetched}>
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
							d="M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.14 0M1.394 9.393c5.857-5.858 15.355-5.858 21.213 0"
						/>
					</svg>
					P2P
				</button>
			{/if}
		</div>

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

		{#if createdBy.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Created by</h3>
				<p class="text-sm">{createdBy.join(', ')}</p>
			</div>
		{/if}

		{#if networks.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Networks</h3>
				<p class="text-sm">{networks.join(', ')}</p>
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

		{#if tvSeasonDetails.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">
					Seasons &amp; Episodes
					{#if tvMatchedSeasons.hasComplete}
						<span class="ml-1 badge badge-xs badge-success">Complete</span>
					{/if}
				</h3>
				<div class="flex flex-col gap-1">
					{#each tvSeasonDetails as season (season.seasonNumber)}
						{@const isExpanded = expandedSeason === season.seasonNumber}
						{@const seasonMatch = tvMatchedSeasons.seasons.get(season.seasonNumber)}
						{@const hasSeasonPack = seasonMatch?.has(-1) ?? false}
						<div
							class={classNames('rounded', {
								'bg-success/10 ring-1 ring-success/30': hasSeasonPack,
								'bg-base-200': !hasSeasonPack
							})}
						>
							<button
								class={classNames(
									'flex w-full items-center justify-between px-2 py-1.5 text-left text-sm',
									{
										'hover:bg-success/20': hasSeasonPack,
										'hover:bg-base-300': !hasSeasonPack
									}
								)}
								onclick={() => {
									expandedSeason = isExpanded ? null : season.seasonNumber;
								}}
							>
								<span class="flex min-w-0 items-center gap-1.5">
									{#if hasSeasonPack}
										<span class="text-xs text-success" title="Season pack found">●</span>
									{:else if seasonMatch}
										<span class="text-xs text-warning" title="Some episodes found">◐</span>
									{/if}
									<span class="truncate font-medium">{season.name}</span>
								</span>
								<span class="flex shrink-0 items-center gap-1">
									{#if seasonMatch}
										{@const epCount = [...seasonMatch].filter((e) => e !== -1).length}
										{#if epCount > 0}
											<span class="text-xs text-success">{epCount}/{season.episodes.length}</span>
										{:else}
											<span class="text-xs opacity-50">{season.episodes.length} ep</span>
										{/if}
									{:else}
										<span class="text-xs opacity-50">{season.episodes.length} ep</span>
									{/if}
									<span class="text-xs opacity-40">{isExpanded ? '▲' : '▼'}</span>
								</span>
							</button>
							{#if isExpanded}
								<div class="border-t border-base-300 px-2 py-1">
									{#each season.episodes as ep (ep.episodeNumber)}
										{@const epMatched = seasonMatch?.has(ep.episodeNumber) ?? false}
										<div
											class={classNames('flex items-baseline gap-2 py-0.5', {
												'text-success': epMatched
											})}
										>
											<span
												class={classNames('shrink-0 font-mono text-xs', {
													'opacity-40': !epMatched
												})}
											>
												{#if epMatched}●{/if}
												E{String(ep.episodeNumber).padStart(2, '0')}
											</span>
											<span class="min-w-0 truncate text-xs">{ep.name}</span>
											{#if ep.runtime}
												<span class="ml-auto shrink-0 text-xs opacity-40">
													{ep.runtime}m
												</span>
											{/if}
										</div>
									{/each}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		{/if}

		{#if backdropUrl || posterUrl}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">
					Primary Images
				</h3>
				<div class="flex flex-col gap-2">
					{#if backdropUrl}
						<div>
							<p class="mb-0.5 font-mono text-xs opacity-40">backdrop_path</p>
							<img
								src={backdropUrl}
								alt="{title} backdrop"
								class="aspect-video w-full rounded object-cover"
								loading="lazy"
							/>
						</div>
					{/if}
					{#if posterUrl}
						<div>
							<p class="mb-0.5 font-mono text-xs opacity-40">poster_path</p>
							<img
								src={posterUrl}
								alt="{title} poster"
								class="aspect-[2/3] w-32 rounded object-cover"
								loading="lazy"
							/>
						</div>
					{/if}
				</div>
			</div>
		{/if}

		{#if images.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">All Images</h3>
				<div class="grid grid-cols-3 gap-1">
					{#each images as image}
						<a href={image.fullUrl} target="_blank" rel="noopener noreferrer">
							<img
								src={image.thumbnailUrl}
								alt="{title} image"
								class={classNames(
									'w-full rounded object-cover transition-opacity hover:opacity-80',
									{
										'aspect-video': image.width > image.height,
										'aspect-[2/3]': image.width <= image.height
									}
								)}
								loading="lazy"
							/>
						</a>
					{/each}
				</div>
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
						<tr
							><td class="font-medium opacity-60">Path</td><td class="break-all"
								>{libraryItem.path}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Extension</td><td>{libraryItem.extension}</td></tr
						>
						<tr
							><td class="font-medium opacity-60">Media Type</td><td>{libraryItem.mediaTypeId}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Category</td><td
								>{libraryItem.categoryId ?? '—'}</td
							></tr
						>
						<tr><td class="font-medium opacity-60">Created</td><td>{libraryItem.createdAt}</td></tr>
						{#each Object.entries(libraryItem.links) as [service, link]}
							<tr
								><td class="font-medium opacity-60">Link: {service}</td><td class="break-all"
									>{link.serviceId}</td
								></tr
							>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}

		{#if relatedData?.library}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Library</h3>
				<table class="table table-xs">
					<tbody>
						<tr
							><td class="font-medium opacity-60">Name</td><td class="break-all"
								>{relatedData.library.name}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Path</td><td class="break-all"
								>{relatedData.library.path}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Media Types</td><td
								>{relatedData.library.mediaTypes}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Created</td><td
								>{relatedData.library.createdAt}</td
							></tr
						>
					</tbody>
				</table>
			</div>
		{/if}

		{#if relatedData?.links && relatedData.links.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Item Links</h3>
				<table class="table table-xs">
					<tbody>
						{#each relatedData.links as link}
							<tr
								><td class="font-medium opacity-60">{link.service}</td><td class="break-all"
									>{link.serviceId}</td
								></tr
							>
							{#if link.seasonNumber != null}
								<tr
									><td class="pl-4 font-medium opacity-60">Season</td><td>{link.seasonNumber}</td
									></tr
								>
							{/if}
							{#if link.episodeNumber != null}
								<tr
									><td class="pl-4 font-medium opacity-60">Episode</td><td>{link.episodeNumber}</td
									></tr
								>
							{/if}
							<tr><td class="pl-4 font-medium opacity-60">Linked</td><td>{link.createdAt}</td></tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}

		{#if relatedData?.fetchCache}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Fetch Cache</h3>
				<table class="table table-xs">
					<tbody>
						<tr
							><td class="font-medium opacity-60">TMDB ID</td><td
								>{relatedData.fetchCache.tmdbId}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Media Type</td><td
								>{relatedData.fetchCache.mediaType}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Created</td><td
								>{relatedData.fetchCache.createdAt}</td
							></tr
						>
						{#if relatedData.fetchCache.candidate.name}
							<tr
								><td class="font-medium opacity-60">Torrent</td><td class="break-all"
									>{relatedData.fetchCache.candidate.name}</td
								></tr
							>
						{/if}
						{#if relatedData.fetchCache.candidate.infoHash}
							<tr
								><td class="font-medium opacity-60">Info Hash</td><td
									class="font-mono text-[10px] break-all"
									>{relatedData.fetchCache.candidate.infoHash}</td
								></tr
							>
						{/if}
						{#if relatedData.fetchCache.candidate.size}
							<tr
								><td class="font-medium opacity-60">Size</td><td
									>{formatBytes(Number(relatedData.fetchCache.candidate.size))}</td
								></tr
							>
						{/if}
						{#if relatedData.fetchCache.candidate.seeds != null}
							<tr
								><td class="font-medium opacity-60">Seeds</td><td
									>{relatedData.fetchCache.candidate.seeds}</td
								></tr
							>
						{/if}
						{#if relatedData.fetchCache.candidate.peers != null}
							<tr
								><td class="font-medium opacity-60">Peers</td><td
									>{relatedData.fetchCache.candidate.peers}</td
								></tr
							>
						{/if}
						{#if relatedData.fetchCache.candidate.magnetUrl}
							<tr
								><td class="font-medium opacity-60">Magnet</td><td
									class="font-mono text-[10px] break-all"
									>{relatedData.fetchCache.candidate.magnetUrl}</td
								></tr
							>
						{/if}
					</tbody>
				</table>
			</div>
		{/if}

		{#if relatedData?.torrentDownload}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">
					Torrent Download
				</h3>
				<table class="table table-xs">
					<tbody>
						<tr
							><td class="font-medium opacity-60">Name</td><td class="break-all"
								>{relatedData.torrentDownload.name}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Info Hash</td><td
								class="font-mono text-[10px] break-all">{relatedData.torrentDownload.infoHash}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">State</td><td
								>{relatedData.torrentDownload.state}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Progress</td><td
								>{formatProgress(relatedData.torrentDownload.progress)}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Size</td><td
								>{formatBytes(relatedData.torrentDownload.size)}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">DL Speed</td><td
								>{formatBytes(relatedData.torrentDownload.downloadSpeed)}/s</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">UL Speed</td><td
								>{formatBytes(relatedData.torrentDownload.uploadSpeed)}/s</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Peers</td><td
								>{relatedData.torrentDownload.peers}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Seeds</td><td
								>{relatedData.torrentDownload.seeds}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Source</td><td
								>{relatedData.torrentDownload.source}</td
							></tr
						>
						{#if relatedData.torrentDownload.outputPath}
							<tr
								><td class="font-medium opacity-60">Output</td><td class="break-all"
									>{relatedData.torrentDownload.outputPath}</td
								></tr
							>
						{/if}
						<tr
							><td class="font-medium opacity-60">Added</td><td
								>{new Date(relatedData.torrentDownload.addedAt * 1000).toLocaleString()}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Created</td><td
								>{relatedData.torrentDownload.createdAt}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Updated</td><td
								>{relatedData.torrentDownload.updatedAt}</td
							></tr
						>
					</tbody>
				</table>
			</div>
		{/if}

		{#if relatedData?.tmdbCache}
			<div>
				<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">TMDB Cache</h3>
				<table class="table table-xs">
					<tbody>
						<tr
							><td class="font-medium opacity-60">TMDB ID</td><td>{relatedData.tmdbCache.tmdbId}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Fetched</td><td
								>{relatedData.tmdbCache.fetchedAt}</td
							></tr
						>
						<tr
							><td class="font-medium opacity-60">Title</td><td class="break-all"
								>{relatedData.tmdbCache.data.title ?? relatedData.tmdbCache.data.name ?? '—'}</td
							></tr
						>
						{#if relatedData.tmdbCache.data.release_date}
							<tr
								><td class="font-medium opacity-60">Release</td><td
									>{relatedData.tmdbCache.data.release_date}</td
								></tr
							>
						{/if}
						{#if relatedData.tmdbCache.data.runtime}
							<tr
								><td class="font-medium opacity-60">Runtime</td><td
									>{relatedData.tmdbCache.data.runtime} min</td
								></tr
							>
						{/if}
						{#if relatedData.tmdbCache.data.status}
							<tr
								><td class="font-medium opacity-60">Status</td><td
									>{relatedData.tmdbCache.data.status}</td
								></tr
							>
						{/if}
						{#if relatedData.tmdbCache.data.original_language}
							<tr
								><td class="font-medium opacity-60">Language</td><td
									>{relatedData.tmdbCache.data.original_language}</td
								></tr
							>
						{/if}
						{#if relatedData.tmdbCache.data.imdb_id}
							<tr
								><td class="font-medium opacity-60">IMDB ID</td><td
									>{relatedData.tmdbCache.data.imdb_id}</td
								></tr
							>
						{/if}
					</tbody>
				</table>
			</div>
		{/if}
	</div>
</div>
