<script lang="ts">
	import type { LibraryFile } from 'ui-lib/types/library.type';
	import type { TMDBMovie, TMDBTvShow, TMDBTvShowDetails, TMDBEpisode } from 'addons/tmdb/types';
	import { getPosterUrl, extractYear } from 'addons/tmdb/transform';

	type TmdbType = 'movie' | 'tv';

	interface EpisodeMatch {
		file: LibraryFile;
		seasonNumber: number;
		episodeNumber: number;
		episodeName: string;
		matched: boolean;
	}

	interface Props {
		file: LibraryFile;
		files?: LibraryFile[];
		type: TmdbType;
		onlink: (
			tmdbId: number,
			seasonNumber: number | null,
			episodeNumber: number | null,
			type: TmdbType
		) => void;
		onlinkall?: (
			tmdbId: number,
			matches: Array<{ file: LibraryFile; seasonNumber: number; episodeNumber: number }>,
			type: TmdbType
		) => void;
		onclose: () => void;
	}

	let { file, files = [], type, onlink, onlinkall, onclose }: Props = $props();

	let query = $state(cleanFilename(file.name));
	let searching = $state(false);
	let movieResults: TMDBMovie[] = $state([]);
	let tvResults: TMDBTvShow[] = $state([]);
	let error: string | null = $state(null);

	// Episode matching state
	let selectedShow: TMDBTvShow | null = $state(null);
	let loadingEpisodes = $state(false);
	let episodeMatches: EpisodeMatch[] = $state([]);

	function cleanFilename(name: string): string {
		return name
			.replace(/\.[^.]+$/, '')
			.replace(/[._]/g, ' ')
			.replace(/\s*[\[(].*?[\])]\s*/g, ' ')
			.replace(/\b(720|1080|2160|480)p?\b/gi, '')
			.replace(
				/\b(x264|x265|h264|h265|hevc|avc|bluray|bdrip|brrip|webrip|web-dl|hdtv|dvdrip|hdrip)\b/gi,
				''
			)
			.replace(/\b(aac|ac3|dts|mp3|flac|atmos|truehd)\b/gi, '')
			.replace(/\b(s\d{1,2}e\d{1,2})\b/gi, '')
			.replace(/\s{2,}/g, ' ')
			.trim();
	}

	function parseEpisodeFromFilename(name: string): { season: number; episode: number } | null {
		const match = name.match(/[Ss](\d{1,2})[Ee](\d{1,2})/);
		if (match) {
			return { season: parseInt(match[1], 10), episode: parseInt(match[2], 10) };
		}
		return null;
	}

	async function search() {
		if (!query.trim()) return;
		searching = true;
		error = null;
		movieResults = [];
		tvResults = [];
		selectedShow = null;
		episodeMatches = [];

		try {
			const endpoint = type === 'movie' ? '/api/tmdb/search/movies' : '/api/tmdb/search/tv';
			const params = new URLSearchParams({ q: query.trim() });
			const res = await fetch(`${endpoint}?${params}`);
			const data = await res.json();
			if (!res.ok) {
				error = data.error ?? 'Search failed';
				return;
			}
			if (type === 'movie') {
				movieResults = data.results ?? [];
			} else {
				tvResults = data.results ?? [];
			}
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			searching = false;
		}
	}

	function selectMovie(movie: TMDBMovie) {
		onlink(movie.id, null, null, 'movie');
	}

	async function selectTvShow(show: TMDBTvShow) {
		const allFiles = files.length > 0 ? files : [file];
		const hasParsableFiles = allFiles.some((f) => parseEpisodeFromFilename(f.name) !== null);

		if (!hasParsableFiles || allFiles.length <= 1) {
			onlink(show.id, null, null, 'tv');
			return;
		}

		selectedShow = show;
		loadingEpisodes = true;
		error = null;
		episodeMatches = [];

		try {
			// Fetch show details to get seasons list
			const showRes = await fetch(`/api/tmdb/tv/${show.id}`);
			if (!showRes.ok) {
				error = 'Failed to fetch show details';
				return;
			}
			const showDetails: TMDBTvShowDetails = await showRes.json();
			const seasons = (showDetails.seasons ?? []).filter((s) => s.season_number > 0);

			// Determine which seasons we need based on files
			const neededSeasons = new Set<number>();
			for (const f of allFiles) {
				const parsed = parseEpisodeFromFilename(f.name);
				if (parsed) neededSeasons.add(parsed.season);
			}

			// Fetch episodes for needed seasons
			const episodeMap = new Map<string, TMDBEpisode>();
			await Promise.all(
				seasons
					.filter((s) => neededSeasons.has(s.season_number))
					.map(async (s) => {
						const seasonRes = await fetch(`/api/tmdb/tv/${show.id}/season/${s.season_number}`);
						if (!seasonRes.ok) return;
						const seasonData = await seasonRes.json();
						const episodes: TMDBEpisode[] = seasonData.episodes ?? [];
						for (const ep of episodes) {
							episodeMap.set(`${ep.season_number}-${ep.episode_number}`, ep);
						}
					})
			);

			// Match files to episodes
			episodeMatches = allFiles.map((f) => {
				const parsed = parseEpisodeFromFilename(f.name);
				if (!parsed) {
					return { file: f, seasonNumber: 1, episodeNumber: 1, episodeName: '', matched: false };
				}
				const ep = episodeMap.get(`${parsed.season}-${parsed.episode}`);
				return {
					file: f,
					seasonNumber: parsed.season,
					episodeNumber: parsed.episode,
					episodeName: ep?.name ?? '',
					matched: ep !== undefined
				};
			});
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
			selectedShow = null;
		} finally {
			loadingEpisodes = false;
		}
	}

	function confirmMatches() {
		if (!selectedShow) return;
		const matched = episodeMatches.filter((m) => m.matched);
		if (onlinkall && matched.length > 0) {
			onlinkall(
				selectedShow.id,
				matched.map((m) => ({
					file: m.file,
					seasonNumber: m.seasonNumber,
					episodeNumber: m.episodeNumber
				})),
				'tv'
			);
		} else {
			// Fall back: link the trigger file only
			onlink(selectedShow.id, null, null, 'tv');
		}
	}

	function backToResults() {
		selectedShow = null;
		episodeMatches = [];
		error = null;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			search();
		}
	}
</script>

<div class="modal-open modal">
	<div class="modal-box max-w-2xl">
		<button class="btn absolute top-2 right-2 btn-circle btn-ghost btn-sm" onclick={onclose}>
			&times;
		</button>

		<h3 class="text-lg font-bold">Link {type === 'movie' ? 'Movie' : 'TV Show'}</h3>
		<p class="mt-1 truncate text-sm opacity-60" title={file.name}>{file.name}</p>

		{#if selectedShow}
			<!-- Episode matching view -->
			<div class="mt-3 flex items-center gap-2">
				<button class="btn btn-ghost btn-xs" onclick={backToResults}>&larr; Back</button>
				<span class="text-sm font-medium">{selectedShow.name}</span>
				{#if selectedShow.first_air_date}
					<span class="text-xs opacity-50">{extractYear(selectedShow.first_air_date)}</span>
				{/if}
			</div>

			{#if loadingEpisodes}
				<div class="flex justify-center py-8">
					<span class="loading loading-md loading-spinner"></span>
				</div>
			{:else if error}
				<div class="mt-3 rounded-lg bg-error/10 px-3 py-2 text-sm text-error">{error}</div>
			{:else}
				{@const matched = episodeMatches.filter((m) => m.matched)}
				{@const unmatched = episodeMatches.filter((m) => !m.matched)}

				<div class="mt-3 max-h-80 overflow-y-auto rounded-lg bg-base-100">
					<table class="table w-full table-xs">
						<thead class="sticky top-0 bg-base-100">
							<tr>
								<th>File</th>
								<th class="w-20">Episode</th>
								<th class="w-32">Title</th>
								<th class="w-16">Status</th>
							</tr>
						</thead>
						<tbody>
							{#each episodeMatches as m (m.file.path)}
								<tr>
									<td class="max-w-xs truncate font-mono text-xs opacity-70" title={m.file.name}>
										{m.file.name}
									</td>
									<td class="text-xs">
										{#if m.matched}
											S{String(m.seasonNumber).padStart(2, '0')}E{String(m.episodeNumber).padStart(
												2,
												'0'
											)}
										{:else}
											—
										{/if}
									</td>
									<td class="max-w-xs truncate text-xs opacity-70">{m.episodeName}</td>
									<td>
										{#if m.matched}
											<span class="badge badge-xs badge-success">matched</span>
										{:else}
											<span class="badge badge-xs badge-warning">skip</span>
										{/if}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>

				<div class="mt-3 flex items-center justify-between">
					<span class="text-xs opacity-50">
						{matched.length} matched · {unmatched.length} skipped
					</span>
					<div class="flex gap-2">
						<button class="btn btn-ghost btn-sm" onclick={onclose}>Cancel</button>
						<button
							class="btn btn-sm btn-primary"
							onclick={confirmMatches}
							disabled={matched.length === 0}
						>
							Link {matched.length} file{matched.length !== 1 ? 's' : ''}
						</button>
					</div>
				</div>
			{/if}
		{:else}
			<!-- Search view -->
			<div class="join mt-4 w-full">
				<input
					type="text"
					class="input-bordered input input-sm join-item w-full"
					placeholder="Search {type === 'movie' ? 'movies' : 'TV shows'}..."
					bind:value={query}
					onkeydown={handleKeydown}
				/>
				<button
					class="btn join-item btn-sm btn-primary"
					onclick={search}
					disabled={searching || !query.trim()}
				>
					{#if searching}
						<span class="loading loading-xs loading-spinner"></span>
					{:else}
						Search
					{/if}
				</button>
			</div>

			{#if error}
				<div class="mt-3 rounded-lg bg-error/10 px-3 py-2 text-sm text-error">{error}</div>
			{/if}

			<div class="mt-4 max-h-80 overflow-y-auto">
				{#if searching}
					<div class="flex justify-center py-8">
						<span class="loading loading-md loading-spinner"></span>
					</div>
				{:else if type === 'movie' && movieResults.length > 0}
					<div class="flex flex-col gap-2">
						{#each movieResults as movie (movie.id)}
							<button
								class="flex items-center gap-3 rounded-lg bg-base-200 p-3 text-left transition-colors hover:bg-base-300"
								onclick={() => selectMovie(movie)}
							>
								<div class="h-16 w-11 flex-shrink-0 overflow-hidden rounded bg-base-300">
									{#if movie.poster_path}
										<img
											src={getPosterUrl(movie.poster_path, 'w185')}
											alt={movie.title}
											class="h-full w-full object-cover"
										/>
									{/if}
								</div>
								<div class="flex-1 overflow-hidden">
									<p class="truncate text-sm font-medium">{movie.title}</p>
									<p class="text-xs opacity-60">
										{extractYear(movie.release_date)}
										{#if movie.vote_average}
											&middot; {movie.vote_average.toFixed(1)}
										{/if}
									</p>
									{#if movie.overview}
										<p class="mt-1 line-clamp-2 text-xs opacity-50">{movie.overview}</p>
									{/if}
								</div>
							</button>
						{/each}
					</div>
				{:else if type === 'tv' && tvResults.length > 0}
					<div class="flex flex-col gap-2">
						{#each tvResults as show (show.id)}
							<button
								class="flex items-center gap-3 rounded-lg bg-base-200 p-3 text-left transition-colors hover:bg-base-300"
								onclick={() => selectTvShow(show)}
							>
								<div class="h-16 w-11 flex-shrink-0 overflow-hidden rounded bg-base-300">
									{#if show.poster_path}
										<img
											src={getPosterUrl(show.poster_path, 'w185')}
											alt={show.name}
											class="h-full w-full object-cover"
										/>
									{/if}
								</div>
								<div class="flex-1 overflow-hidden">
									<p class="truncate text-sm font-medium">{show.name}</p>
									<p class="text-xs opacity-60">
										{extractYear(show.first_air_date)}
										{#if show.vote_average}
											&middot; {show.vote_average.toFixed(1)}
										{/if}
										{#if show.number_of_seasons}
											&middot; {show.number_of_seasons} season{show.number_of_seasons !== 1
												? 's'
												: ''}
										{/if}
									</p>
									{#if show.overview}
										<p class="mt-1 line-clamp-2 text-xs opacity-50">{show.overview}</p>
									{/if}
								</div>
							</button>
						{/each}
					</div>
				{:else if ((type === 'movie' && movieResults.length === 0) || (type === 'tv' && tvResults.length === 0)) && !searching && query.trim()}
					<div class="py-8 text-center">
						<p class="text-sm opacity-50">No results found</p>
					</div>
				{/if}
			</div>
		{/if}
	</div>
	<div class="modal-backdrop" onclick={onclose}></div>
</div>
