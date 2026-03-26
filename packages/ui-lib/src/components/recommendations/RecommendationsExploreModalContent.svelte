<script lang="ts">
	import { onMount } from 'svelte';
	import classNames from 'classnames';
	import { recommendationsService } from 'ui-lib/services/recommendations.service';
	import type { TopRecommendedMovieDetail } from 'ui-lib/types/recommendations.type';
	import { getPosterUrl, getBackdropUrl, extractYear } from 'addons/tmdb/transform';

	let movies = $state<TopRecommendedMovieDetail[]>([]);
	let loading = $state(false);
	let selectedIndex = $state<number | null>(null);

	async function loadMovies() {
		loading = true;
		try {
			movies = await recommendationsService.getTopMoviesDetail();
		} catch {
			/* best-effort */
		} finally {
			loading = false;
		}
	}

	function posterUrl(data: Record<string, unknown> | null): string | null {
		if (!data) return null;
		return getPosterUrl(data.poster_path as string | null);
	}

	function backdropUrl(data: Record<string, unknown> | null): string | null {
		if (!data) return null;
		return getBackdropUrl(data.backdrop_path as string | null);
	}

	function year(data: Record<string, unknown> | null): string {
		if (!data) return '';
		return extractYear(data.release_date as string | undefined);
	}

	function rating(data: Record<string, unknown> | null): string {
		if (!data) return '—';
		const val = data.vote_average as number | undefined;
		return val != null ? val.toFixed(1) : '—';
	}

	function overview(data: Record<string, unknown> | null): string {
		if (!data) return '';
		return (data.overview as string) ?? '';
	}

	function scrollToCard(index: number) {
		selectedIndex = index;
		const el = document.getElementById(`rec-card-${index}`);
		if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' });
	}

	onMount(() => {
		loadMovies();
	});
</script>

<div class="flex max-h-[80vh] flex-col gap-4 overflow-hidden">
	<div class="flex items-center justify-between">
		<h2 class="text-lg font-bold">Explore Recommendations</h2>
		<button class="btn btn-ghost btn-sm" onclick={loadMovies} disabled={loading}>
			{#if loading}
				<span class="loading loading-xs loading-spinner"></span>
			{/if}
			Refresh
		</button>
	</div>

	{#if loading && movies.length === 0}
		<div class="flex justify-center py-12">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if movies.length === 0}
		<p class="py-12 text-center text-sm text-base-content/50">
			No recommendations yet. Use the Recs modal to enqueue movies first.
		</p>
	{:else}
		<div class="grid min-h-0 flex-1 grid-cols-2 grid-rows-1 gap-4">
			<!-- Left: Top Recommended Movies table (same as Recs modal right panel) -->
			<div class="min-h-0 flex flex-col gap-2 overflow-y-auto">
				<h3 class="text-sm font-semibold">Top Recommended Movies</h3>
				<table class="table table-xs">
					<thead>
						<tr>
							<th>#</th>
							<th>Title</th>
							<th>TMDB ID</th>
							<th>Count</th>
							<th>Level</th>
						</tr>
					</thead>
					<tbody>
						{#each movies as movie, i (movie.tmdbId)}
							<tr
								class={classNames('cursor-pointer', {
									'bg-primary/20': selectedIndex === i,
									'hover:bg-base-200': selectedIndex !== i
								})}
								onclick={() => scrollToCard(i)}
							>
								<td class="text-base-content/40">{i + 1}</td>
								<td class="max-w-48 truncate">{movie.title ?? '—'}</td>
								<td class="font-mono text-xs">{movie.tmdbId}</td>
								<td class="font-semibold">{movie.count}</td>
								<td><span class="badge badge-ghost badge-xs">L{movie.minLevel}</span></td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			<!-- Right: Detail cards -->
			<div class="min-h-0 flex flex-col gap-6 overflow-y-auto pr-1">
				{#each movies as movie, i (movie.tmdbId)}
					<div
						id={`rec-card-${i}`}
						class={classNames('overflow-hidden rounded-xl bg-base-200', {
							'ring-2 ring-primary': selectedIndex === i
						})}
					>
						{#if backdropUrl(movie.data)}
							<img src={backdropUrl(movie.data)} alt="" class="h-44 w-full object-cover" />
						{/if}
						<div class="flex gap-4 p-4">
							{#if posterUrl(movie.data)}
								<img
									src={posterUrl(movie.data)}
									alt=""
									class="h-36 w-24 flex-shrink-0 rounded-lg object-cover shadow-md"
								/>
							{/if}
							<div class="min-w-0 flex-1">
								<h3 class="text-base font-bold">{movie.title ?? '—'}</h3>
								<div class="mt-1 flex flex-wrap items-center gap-2 text-sm text-base-content/60">
									<span>{year(movie.data)}</span>
									<span>&middot;</span>
									<span>{rating(movie.data)} / 10</span>
									<span>&middot;</span>
									<span class="badge badge-ghost badge-sm">
										Recommended {movie.count}x
									</span>
									<span class="badge badge-ghost badge-sm">
										Level {movie.minLevel}
									</span>
								</div>
								{#if overview(movie.data)}
									<p class="mt-2 text-sm leading-relaxed text-base-content/70">
										{overview(movie.data)}
									</p>
								{/if}
								{#if movie.sources.length > 0}
									<p class="mt-2 text-xs text-base-content/50">
										Recommended from: {movie.sources
											.map((s) => s.title ?? `TMDB #${s.tmdbId}`)
											.join(', ')}
									</p>
								{/if}
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>
