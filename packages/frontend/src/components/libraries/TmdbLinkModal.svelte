<script lang="ts">
	import type { LibraryFile } from '$types/library.type';
	import type { TMDBMovie, TMDBTvShow } from 'tmdb/types';
	import { getPosterUrl, extractYear } from 'tmdb/transform';

	type TmdbType = 'movie' | 'tv';

	interface Props {
		file: LibraryFile;
		type: TmdbType;
		onlink: (
			tmdbId: number,
			seasonNumber: number | null,
			episodeNumber: number | null,
			type: TmdbType
		) => void;
		onclose: () => void;
	}

	let { file, type, onlink, onclose }: Props = $props();

	let query = $state(cleanFilename(file.name));
	let searching = $state(false);
	let movieResults: TMDBMovie[] = $state([]);
	let tvResults: TMDBTvShow[] = $state([]);
	let error: string | null = $state(null);

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

	async function search() {
		if (!query.trim()) return;
		searching = true;
		error = null;
		movieResults = [];
		tvResults = [];

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

	function selectTvShow(show: TMDBTvShow) {
		onlink(show.id, null, null, 'tv');
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
	</div>
	<div class="modal-backdrop" onclick={onclose}></div>
</div>
