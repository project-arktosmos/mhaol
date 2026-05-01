<script lang="ts">
	import type { TMDBTvShow } from 'addons/tmdb/types';
	import { getPosterUrl, extractYear } from 'addons/tmdb/transform';
	import { fetchRaw } from '$transport/fetch-helpers';

	interface Props {
		showName: string;
		onmatch: (tmdbId: number) => void;
		onclose: () => void;
	}

	let { showName, onmatch, onclose }: Props = $props();

	let query = $state(showName);
	let searching = $state(false);
	let results: TMDBTvShow[] = $state([]);
	let error: string | null = $state(null);
	let searched = $state(false);

	async function search() {
		if (!query.trim()) return;
		searching = true;
		error = null;
		results = [];

		try {
			const params = new URLSearchParams({ q: query.trim() });
			const res = await fetchRaw(`/api/tmdb/search/tv?${params}`);
			const data = await res.json();
			if (!res.ok) {
				error = data.error ?? 'Search failed';
				return;
			}
			results = data.results ?? [];
		} catch (e) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			searching = false;
			searched = true;
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			search();
		}
	}

	$effect(() => {
		if (showName) search();
	});
</script>

<div class="modal-open modal">
	<div class="modal-box max-w-2xl">
		<button class="btn absolute top-2 right-2 btn-circle btn-ghost btn-sm" onclick={onclose}>
			&times;
		</button>

		<h3 class="text-lg font-bold">Match TV Show Metadata</h3>
		<p class="mt-1 text-sm opacity-60">Search TMDB to find metadata for "{showName}"</p>

		<div class="join mt-4 w-full">
			<input
				type="text"
				class="input-bordered input input-sm join-item w-full"
				placeholder="Search TV shows..."
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

		<div class="mt-4 max-h-96 overflow-y-auto">
			{#if searching}
				<div class="flex justify-center py-8">
					<span class="loading loading-md loading-spinner"></span>
				</div>
			{:else if results.length > 0}
				<div class="flex flex-col gap-2">
					{#each results as show (show.id)}
						<button
							class="flex items-center gap-3 rounded-lg bg-base-200 p-3 text-left transition-colors hover:bg-base-300"
							onclick={() => onmatch(show.id)}
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
			{:else if searched && !searching && query.trim()}
				<div class="py-8 text-center">
					<p class="text-sm opacity-50">No results found</p>
				</div>
			{/if}
		</div>
	</div>
	<div class="modal-backdrop" onclick={onclose}></div>
</div>
