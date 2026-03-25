<script lang="ts">
	import classNames from 'classnames';
	import DetailPageLayout from 'ui-lib/components/core/DetailPageLayout.svelte';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	import { playerService } from 'ui-lib/services/player.service';
	import type {
		DisplayTMDBTvShow,
		DisplayTMDBTvShowDetails,
		DisplayTMDBSeasonDetails,
		DisplayTMDBImage
	} from 'addons/tmdb/types';
	import {
		formatBytes,
		formatSpeed,
		formatEta,
		getStateLabel,
		getStateColor
	} from 'ui-lib/types/torrent.type';
	import type { TorrentState } from 'ui-lib/types/torrent.type';

	export interface LibraryEpisodeFile {
		seasonNumber: number;
		episodeNumber: number;
		name: string;
		path: string;
	}

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
		torrentStatus: {
			state: TorrentState;
			progress: number;
			size: number;
			downloadSpeed: number;
			uploadSpeed: number;
			peers: number;
			seeds: number;
			eta: number | null;
		} | null;
		fetchedTorrent: { name: string; quality: string; languages: string } | null;
		tvMatchedSeasons: {
			hasComplete: boolean;
			seasons: Map<number, Set<number>>;
		};
		libraryFiles?: LibraryEpisodeFile[];
		resyncing?: boolean;
		onfetch: () => void;
		ondownload: () => void;
		onp2pstream: () => void;
		onshowsearch: () => void;
		onplayfile?: (file: LibraryEpisodeFile) => void;
		onresync?: () => void;
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
		torrentStatus,
		fetchedTorrent,
		tvMatchedSeasons,
		libraryFiles = [],
		resyncing = false,
		onfetch,
		ondownload,
		onp2pstream,
		onshowsearch,
		onplayfile,
		onresync,
		onback
	}: Props = $props();

	let hasLibrary = $derived(libraryFiles.length > 0);

	// Index library files by "season-episode" for quick lookup
	let libraryFileMap = $derived(
		new Map(libraryFiles.map((f) => [`${f.seasonNumber}-${f.episodeNumber}`, f]))
	);

	// Group library files by season for the sidebar listing
	let libraryFilesBySeason = $derived.by(() => {
		const grouped = new Map<number, LibraryEpisodeFile[]>();
		for (const f of libraryFiles) {
			const existing = grouped.get(f.seasonNumber);
			if (existing) {
				existing.push(f);
			} else {
				grouped.set(f.seasonNumber, [f]);
			}
		}
		// Sort episodes within each season
		for (const files of grouped.values()) {
			files.sort((a, b) => a.episodeNumber - b.episodeNumber);
		}
		return grouped;
	});

	let expandedSeason = $state<number | null>(null);
	let streamingP2p = $state(false);

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

	let dlState = $derived(torrentStatus?.state ?? null);
	let isDownloading = $derived(
		dlState === 'downloading' ||
			dlState === 'initializing' ||
			dlState === 'paused' ||
			dlState === 'checking'
	);
	let isDownloaded = $derived(dlState === 'seeding');
	let downloadButtonDisabled = $derived(!fetched || isDownloading || isDownloaded);
	let dlPercent = $derived(Math.round((torrentStatus?.progress ?? 0) * 100));

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

	{#snippet cellA()}
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

		{#if backdropUrl}
			<img
				src={backdropUrl}
				alt="{title} backdrop"
				class="aspect-video w-full rounded-lg object-cover"
				loading="lazy"
			/>
		{/if}

		{#if hasLibrary}
			<div class="rounded-lg bg-success/10 px-3 py-2 text-sm text-success">
				{libraryFiles.length} file{libraryFiles.length !== 1 ? 's' : ''} in library
			</div>
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

		{#if !hasLibrary}
			<button
				class="btn w-full btn-sm {fetched ? 'btn-ghost' : 'btn-info'}"
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
					class="w-full cursor-pointer rounded-lg bg-base-200 p-2 transition-colors hover:bg-base-300"
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

			{#if fetchedTorrent || torrentStatus}
				<table class="table table-xs">
					<tbody>
						{#if fetchedTorrent}
							<tr>
								<td class="font-medium opacity-60">File</td>
								<td class="break-all">{fetchedTorrent.name}</td>
							</tr>
							{#if fetchedTorrent.quality}
								<tr>
									<td class="font-medium opacity-60">Quality</td>
									<td><span class="badge badge-xs badge-info">{fetchedTorrent.quality}</span></td>
								</tr>
							{/if}
							{#if fetchedTorrent.languages}
								<tr>
									<td class="font-medium opacity-60">Languages</td>
									<td><span class="badge badge-ghost badge-xs">{fetchedTorrent.languages}</span></td
									>
								</tr>
							{/if}
						{/if}
						{#if torrentStatus}
							<tr>
								<td class="font-medium opacity-60">Status</td>
								<td>
									<span class="badge badge-xs badge-{getStateColor(torrentStatus.state)}">
										{getStateLabel(torrentStatus.state)}
									</span>
								</td>
							</tr>
							<tr>
								<td class="font-medium opacity-60">Size</td>
								<td>{formatBytes(torrentStatus.size)}</td>
							</tr>
							{#if isDownloading}
								<tr>
									<td class="font-medium opacity-60">Progress</td>
									<td>
										<div class="flex items-center gap-2">
											<progress class="progress flex-1 progress-info" value={dlPercent} max="100"
											></progress>
											<span class="text-xs font-medium">{dlPercent}%</span>
										</div>
									</td>
								</tr>
								<tr>
									<td class="font-medium opacity-60">Speed</td>
									<td>
										{formatSpeed(torrentStatus.downloadSpeed)} &darr;
										{formatSpeed(torrentStatus.uploadSpeed)} &uarr;
									</td>
								</tr>
								<tr>
									<td class="font-medium opacity-60">Peers</td>
									<td>{torrentStatus.seeds} seeds &middot; {torrentStatus.peers} peers</td>
								</tr>
								{#if torrentStatus.eta !== null}
									<tr>
										<td class="font-medium opacity-60">ETA</td>
										<td>{formatEta(torrentStatus.eta)}</td>
									</tr>
								{/if}
							{/if}
							{#if isDownloaded}
								<tr>
									<td class="font-medium opacity-60">Progress</td>
									<td>
										<div class="flex items-center gap-2">
											<progress class="progress flex-1 progress-success" value="100" max="100"
											></progress>
											<span class="text-xs font-medium">100%</span>
										</div>
									</td>
								</tr>
							{/if}
						{:else if fetchedTorrent}
							<tr>
								<td class="font-medium opacity-60">Status</td>
								<td><span class="badge badge-ghost badge-xs">Not started</span></td>
							</tr>
						{/if}
					</tbody>
				</table>
			{/if}

			<div class="grid grid-cols-2 gap-2">
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
					class="btn col-span-2 btn-sm btn-secondary"
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
			</div>
		{/if}

		{#if tvSeasonDetails.length > 0}
			<div>
				<div class="mb-1 flex items-center justify-between">
					<h3 class="text-xs font-semibold tracking-wide uppercase opacity-50">
						Seasons &amp; Episodes
						{#if hasLibrary}
							<span class="ml-1 badge badge-xs badge-success"
								>{libraryFiles.length} file{libraryFiles.length !== 1 ? 's' : ''}</span
							>
						{:else if tvMatchedSeasons.hasComplete}
							<span class="ml-1 badge badge-xs badge-success">Complete</span>
						{/if}
					</h3>
					{#if onresync}
						<button
							class="btn btn-ghost btn-xs"
							onclick={onresync}
							disabled={resyncing}
							title="Resync library files"
						>
							{#if resyncing}
								<span class="loading loading-xs loading-spinner"></span>
							{:else}
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
										d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
									/>
								</svg>
							{/if}
							Resync
						</button>
					{/if}
				</div>
				<div class="flex flex-col gap-1">
					{#each tvSeasonDetails as season (season.seasonNumber)}
						{@const isExpanded = expandedSeason === season.seasonNumber}
						{@const seasonMatch = tvMatchedSeasons.seasons.get(season.seasonNumber)}
						{@const hasSeasonPack = seasonMatch?.has(-1) ?? false}
						{@const seasonLibFiles = libraryFilesBySeason.get(season.seasonNumber)}
						{@const hasSeasonLib = (seasonLibFiles?.length ?? 0) > 0}
						<div
							class={classNames('rounded', {
								'bg-success/10 ring-1 ring-success/30': hasSeasonLib || hasSeasonPack,
								'bg-base-200': !hasSeasonLib && !hasSeasonPack
							})}
						>
							<button
								class={classNames(
									'flex w-full items-center justify-between px-2 py-1.5 text-left text-sm',
									{
										'hover:bg-success/20': hasSeasonLib || hasSeasonPack,
										'hover:bg-base-300': !hasSeasonLib && !hasSeasonPack
									}
								)}
								onclick={() => (expandedSeason = isExpanded ? null : season.seasonNumber)}
							>
								<span class="flex min-w-0 items-center gap-1.5">
									{#if hasSeasonLib}
										<span class="text-xs text-success" title="Library files available">●</span>
									{:else if hasSeasonPack}
										<span class="text-xs text-success" title="Season pack found">●</span>
									{:else if seasonMatch}
										<span class="text-xs text-warning" title="Some episodes found">◐</span>
									{/if}
									<span class="truncate font-medium">{season.name}</span>
								</span>
								<span class="flex shrink-0 items-center gap-1">
									{#if hasSeasonLib}
										<span class="text-xs text-success"
											>{seasonLibFiles?.length}/{season.episodes.length}</span
										>
									{:else if seasonMatch}
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
										{@const libFile = libraryFileMap.get(
											`${season.seasonNumber}-${ep.episodeNumber}`
										)}
										{@const epMatched =
											libFile != null || (seasonMatch?.has(ep.episodeNumber) ?? false)}
										<div
											class={classNames('group flex items-center gap-2 py-0.5', {
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
											<div class="min-w-0 flex-1">
												{#if libFile}
													<p class="truncate text-xs font-medium">{ep.name}</p>
													<p class="truncate text-xs opacity-50" title={libFile.name}>
														{libFile.name}
													</p>
												{:else}
													<p class="truncate text-xs">{ep.name}</p>
												{/if}
											</div>
											{#if libFile && onplayfile}
												<button
													class="btn shrink-0 opacity-0 btn-ghost btn-xs group-hover:opacity-100"
													onclick={() => onplayfile(libFile)}
													title="Play"
												>
													▶
												</button>
											{:else if ep.runtime}
												<span class="shrink-0 text-xs opacity-40">{ep.runtime}m</span>
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
	{/snippet}
</DetailPageLayout>
