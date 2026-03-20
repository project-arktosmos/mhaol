<script lang="ts">
	import classNames from 'classnames';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';

	let {
		movie = null,
		tvShow = null,
		selected = false,
		onclick
	}: {
		movie?: DisplayTMDBMovie | null;
		tvShow?: DisplayTMDBTvShow | null;
		selected?: boolean;
		onclick?: () => void;
	} = $props();

	let title = $derived(movie?.title ?? tvShow?.name ?? '');
	let year = $derived(movie?.releaseYear ?? tvShow?.firstAirYear ?? '');
	let posterUrl = $derived(movie?.posterUrl ?? tvShow?.posterUrl ?? null);
	let voteAverage = $derived(movie?.voteAverage ?? tvShow?.voteAverage ?? 0);
	let overview = $derived(movie?.overview ?? tvShow?.overview ?? '');
	let genres = $derived(movie?.genres ?? tvShow?.genres ?? []);
</script>

<div
	class={classNames('group card-compact card bg-base-200 shadow-sm', {
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
	<figure class="relative h-48 overflow-hidden bg-base-300">
		{#if posterUrl}
			<img src={posterUrl} alt={title} class="h-full w-full object-cover" loading="lazy" />
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
			{#if tvShow}
				<span class="badge badge-xs badge-info">TV</span>
			{:else}
				<span class="badge badge-xs badge-primary">Movie</span>
			{/if}
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
