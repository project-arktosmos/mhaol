<script lang="ts">
	import classNames from 'classnames';
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import type {
		DisplayTMDBTvShow,
		DisplayTMDBTvShowDetails,
		DisplayTMDBSeasonDetails,
		DisplayTMDBImage
	} from 'addons/tmdb/types';

	interface Props {
		tvShow: DisplayTMDBTvShow;
		tvShowDetails: DisplayTMDBTvShowDetails | null;
		tvSeasonDetails: DisplayTMDBSeasonDetails[];
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
		tvMatchedSeasons: {
			hasComplete: boolean;
			seasons: Map<number, Set<number>>;
		};
		onfetch: () => void;
		ondownload: () => void;
		onstream: () => void;
		onp2pstream: () => void;
		onshowsearch: () => void;
		onback: () => void;
	}

	let {
		tvShow,
		tvShowDetails,
		tvSeasonDetails,
		loading,
		fetching,
		fetched,
		fetchSteps,
		downloadStatus,
		tvMatchedSeasons,
		onfetch,
		ondownload,
		onstream,
		onp2pstream,
		onshowsearch,
		onback
	}: Props = $props();

	let expandedSeason = $state<number | null>(null);

	let title = $derived(tvShow.name);
	let year = $derived(tvShow.firstAirYear ?? '');
	let lastAirYear = $derived(tvShowDetails?.lastAirYear ?? tvShow.lastAirYear ?? null);
	let posterUrl = $derived(tvShow.posterUrl);
	let backdropUrl = $derived(tvShow.backdropUrl);
	let voteAverage = $derived(tvShow.voteAverage ?? 0);
	let voteCount = $derived(tvShow.voteCount ?? 0);
	let overview = $derived(tvShow.overview ?? '');
	let genres = $derived(tvShow.genres ?? []);
	let tagline = $derived(tvShowDetails?.tagline ?? null);
	let cast = $derived(tvShowDetails?.cast ?? []);
	let createdBy = $derived(tvShowDetails?.createdBy ?? []);
	let networks = $derived(tvShowDetails?.networks ?? []);
	let status = $derived(tvShowDetails?.status ?? null);
	let numberOfSeasons = $derived(tvShow.numberOfSeasons ?? tvShowDetails?.numberOfSeasons ?? null);
	let numberOfEpisodes = $derived(
		tvShow.numberOfEpisodes ?? tvShowDetails?.numberOfEpisodes ?? null
	);
	let images: DisplayTMDBImage[] = $derived(tvShowDetails?.images ?? []);
	let heroImageUrl = $derived(images.length > 0 ? images[0].thumbnailUrl : backdropUrl);

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

	{#if heroImageUrl}
		<div class="relative">
			<img src={heroImageUrl} alt={title} class="h-56 w-full rounded-lg object-cover" />
			<div class="absolute inset-0 rounded-lg bg-gradient-to-t from-base-200 to-transparent"></div>
		</div>
	{:else if posterUrl}
		<div class="flex justify-center bg-base-300 p-4">
			<img src={posterUrl} alt={title} class="h-48 rounded-lg object-cover" />
		</div>
	{/if}

	<h1 class="text-xl font-bold">{title}</h1>

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
		<span class="badge badge-sm badge-info">TV</span>
	</div>

	{#if numberOfSeasons != null}
		<p class="text-sm opacity-60">
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
		<button class="btn btn-sm btn-primary" onclick={onstream} disabled={!fetched}>Torrent</button>
		<button class="btn btn-sm btn-secondary" onclick={onp2pstream} disabled={!fetched}>P2P</button>
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
							onclick={() => (expandedSeason = isExpanded ? null : season.seasonNumber)}
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
											<span class="ml-auto shrink-0 text-xs opacity-40">{ep.runtime}m</span>
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

	{#if images.length > 0}
		<div>
			<h3 class="mb-1 text-xs font-semibold tracking-wide uppercase opacity-50">Images</h3>
			<div class="grid grid-cols-3 gap-1">
				{#each images as image}
					<a href={image.fullUrl} target="_blank" rel="noopener noreferrer">
						<img
							src={image.thumbnailUrl}
							alt="{title} image"
							class={classNames('w-full rounded object-cover transition-opacity hover:opacity-80', {
								'aspect-video': image.width > image.height,
								'aspect-[2/3]': image.width <= image.height
							})}
							loading="lazy"
						/>
					</a>
				{/each}
			</div>
		</div>
	{/if}
</DetailPageLayout>
