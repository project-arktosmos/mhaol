<script lang="ts">
	import type {
		DisplayTMDBMovie,
		DisplayTMDBTvShow,
		DisplayTMDBMovieDetails,
		DisplayTMDBTvShowDetails
	} from 'addons/tmdb/types';
	import type { PlayableFile, PlayerConnectionState } from 'frontend/types/player.type';
	import PlayerVideo from 'ui-lib/components/player/PlayerVideo.svelte';
	let {
		movie = null,
		tvShow = null,
		movieDetails = null,
		tvShowDetails = null,
		loading = false,
		fetching = false,
		fetched = false,
		playerFile = null,
		playerConnectionState = 'idle',
		playerPositionSecs = 0,
		playerDurationSecs = 0,
		playerStreamUrl = null,
		playerBuffering = false,
		onfetch,
		ondownload,
		onstream,
		onfullscreen,
		onstopplayer,
		onclose
	}: {
		movie?: DisplayTMDBMovie | null;
		tvShow?: DisplayTMDBTvShow | null;
		movieDetails?: DisplayTMDBMovieDetails | null;
		tvShowDetails?: DisplayTMDBTvShowDetails | null;
		loading?: boolean;
		fetching?: boolean;
		fetched?: boolean;
		playerFile?: PlayableFile | null;
		playerConnectionState?: PlayerConnectionState;
		playerPositionSecs?: number;
		playerDurationSecs?: number | null;
		playerStreamUrl?: string | null;
		playerBuffering?: boolean;
		onfetch?: () => void;
		ondownload?: () => void;
		onstream?: () => void;
		onfullscreen?: () => void;
		onstopplayer?: () => void;
		onclose?: () => void;
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
</script>

<div class="flex h-full flex-col overflow-y-auto">
	<div class="flex items-center justify-between p-3">
		<h2 class="min-w-0 truncate text-sm font-semibold" {title}>{title}</h2>
		{#if onclose}
			<button class="btn btn-square btn-ghost btn-xs shrink-0" onclick={onclose} aria-label="Close">
				&times;
			</button>
		{/if}
	</div>

	{#if playerFile}
		<div class="bg-black">
			<div class="flex items-center justify-between px-2 py-1">
				<p class="min-w-0 truncate text-xs font-semibold text-white" title={playerFile.name}>
					{playerFile.name}
				</p>
				<div class="flex shrink-0 items-center gap-1">
					{#if onfullscreen}
						<button
							class="btn btn-square btn-ghost btn-xs text-white"
							onclick={onfullscreen}
							aria-label="Fullscreen player"
							title="Fullscreen player"
						>
							<svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
								<path stroke-linecap="round" stroke-linejoin="round" d="M4 8V4h4M20 8V4h-4M4 16v4h4M20 16v4h-4" />
							</svg>
						</button>
					{/if}
					{#if onstopplayer}
						<button
							class="btn btn-square btn-ghost btn-xs text-white"
							onclick={onstopplayer}
							aria-label="Close player"
						>
							&times;
						</button>
					{/if}
				</div>
			</div>
			<PlayerVideo
				file={playerFile}
				connectionState={playerConnectionState}
				positionSecs={playerPositionSecs}
				durationSecs={playerDurationSecs}
				streamUrl={playerStreamUrl}
				buffering={playerBuffering}
			/>
		</div>
	{:else if backdropUrl}
		<div class="relative">
			<img src={backdropUrl} alt={title} class="h-40 w-full object-cover" />
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
				{numberOfSeasons} season{numberOfSeasons !== 1 ? 's' : ''}{numberOfEpisodes != null ? `, ${numberOfEpisodes} episodes` : ''}
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

		<div class="flex gap-2">
			{#if onfetch}
				<button class="btn btn-sm btn-info flex-1" onclick={onfetch} disabled={fetching || fetched}>
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
					{fetched ? 'Fetched' : 'Fetch'}
				</button>
			{/if}
			{#if ondownload}
				<button class="btn btn-sm btn-success flex-1" onclick={ondownload} disabled={!fetched}>
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
				</button>
			{/if}
			{#if onstream}
				<button class="btn btn-sm btn-primary flex-1" onclick={onstream} disabled={!fetched}>
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
							d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
						/>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
						/>
					</svg>
					Stream
				</button>
			{/if}
		</div>

		{#if overview}
			<div>
				<h3 class="mb-1 text-xs font-semibold uppercase tracking-wide opacity-50">Overview</h3>
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
				<h3 class="mb-1 text-xs font-semibold uppercase tracking-wide opacity-50">Director</h3>
				<p class="text-sm">{director}</p>
			</div>
		{/if}

		{#if createdBy.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold uppercase tracking-wide opacity-50">Created by</h3>
				<p class="text-sm">{createdBy.join(', ')}</p>
			</div>
		{/if}

		{#if networks.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold uppercase tracking-wide opacity-50">Networks</h3>
				<p class="text-sm">{networks.join(', ')}</p>
			</div>
		{/if}

		{#if cast.length > 0}
			<div>
				<h3 class="mb-1 text-xs font-semibold uppercase tracking-wide opacity-50">Cast</h3>
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
								<div class="flex h-8 w-8 items-center justify-center rounded-full bg-base-300 text-xs opacity-40">
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
	</div>
</div>
