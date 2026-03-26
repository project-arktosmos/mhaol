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

	function posterUrl(data: Record<string, unknown>): string | null {
		return getPosterUrl(data.poster_path as string | null);
	}

	function backdropUrl(data: Record<string, unknown>): string | null {
		return getBackdropUrl(data.backdrop_path as string | null);
	}

	function year(data: Record<string, unknown>): string {
		return extractYear(data.release_date as string | undefined);
	}

	function rating(data: Record<string, unknown>): string {
		const val = data.vote_average as number | undefined;
		return val != null ? val.toFixed(1) : '—';
	}

	function overview(data: Record<string, unknown>): string {
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
		<div class="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_minmax(0,2fr)] gap-4 overflow-hidden">
			<!-- Left: Top movies list -->
			<div class="flex flex-col gap-1 overflow-y-auto">
				{#each movies as movie, i (movie.tmdbId)}
					<button
						class={classNames(
							'flex items-center gap-2 rounded-lg p-2 text-left transition-colors',
							{
								'bg-primary/20': selectedIndex === i,
								'bg-base-200 hover:bg-base-300': selectedIndex !== i
							}
						)}
						onclick={() => scrollToCard(i)}
					>
						<span class="min-w-6 text-right text-xs text-base-content/40">{i + 1}</span>
						{#if posterUrl(movie.data)}
							<img
								src={posterUrl(movie.data)}
								alt=""
								class="h-12 w-8 flex-shrink-0 rounded object-cover"
							/>
						{:else}
							<div class="h-12 w-8 flex-shrink-0 rounded bg-base-300"></div>
						{/if}
						<div class="min-w-0 flex-1">
							<p class="truncate text-sm font-medium">{movie.title ?? '—'}</p>
							<p class="text-xs text-base-content/50">
								{year(movie.data)} &middot; {rating(movie.data)}
							</p>
						</div>
						<span class="badge badge-ghost badge-sm flex-shrink-0">{movie.count}</span>
					</button>
				{/each}
			</div>

			<!-- Right: Detail cards -->
			<div class="flex flex-col gap-6 overflow-y-auto pr-1">
				{#each movies as movie, i (movie.tmdbId)}
					<div
						id={`rec-card-${i}`}
						class={classNames('overflow-hidden rounded-xl bg-base-200', {
							'ring-2 ring-primary': selectedIndex === i
						})}
					>
						{#if backdropUrl(movie.data)}
							<img
								src={backdropUrl(movie.data)}
								alt=""
								class="h-44 w-full object-cover"
							/>
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
								</div>
								{#if overview(movie.data)}
									<p class="mt-2 text-sm leading-relaxed text-base-content/70">
										{overview(movie.data)}
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
