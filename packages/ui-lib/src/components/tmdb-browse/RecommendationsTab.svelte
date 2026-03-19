<script lang="ts">
	import TmdbBrowseCard from './TmdbBrowseCard.svelte';
	import TmdbPagination from './TmdbPagination.svelte';
	import type { DisplayTMDBMovie, DisplayTMDBTvShow } from 'addons/tmdb/types';

	interface LinkedItem {
		tmdbId: number;
		title: string;
		type: 'movie' | 'tv';
	}

	let {
		linkedItems,
		recommendations,
		page,
		totalPages,
		sourceId,
		sourceType,
		loading = false,
		onload
	}: {
		linkedItems: LinkedItem[];
		recommendations: (DisplayTMDBMovie | DisplayTMDBTvShow)[];
		page: number;
		totalPages: number;
		sourceId: number | null;
		sourceType: 'movie' | 'tv' | null;
		loading?: boolean;
		onload: (tmdbId: number, type: 'movie' | 'tv', page: number) => void;
	} = $props();

	let selectedSource = $derived(
		sourceId != null
			? `${sourceType}:${sourceId}`
			: linkedItems.length > 0
				? `${linkedItems[0].type}:${linkedItems[0].tmdbId}`
				: ''
	);

	function handleSourceChange(e: Event) {
		const val = (e.target as HTMLSelectElement).value;
		if (!val) return;
		const [type, id] = val.split(':');
		onload(Number(id), type as 'movie' | 'tv', 1);
	}

	function isMovie(item: DisplayTMDBMovie | DisplayTMDBTvShow): item is DisplayTMDBMovie {
		return 'title' in item;
	}
</script>

{#if linkedItems.length === 0}
	<div class="rounded-lg bg-base-200 p-8 text-center">
		<p class="opacity-50">
			Link movies or TV shows to TMDB in your library to get recommendations.
		</p>
	</div>
{:else}
	<div class="mb-4">
		<label class="text-sm font-medium">
			Based on:
			<select
				class="select-bordered select ml-2 select-sm"
				value={selectedSource}
				onchange={handleSourceChange}
			>
				{#each linkedItems as item}
					<option value="{item.type}:{item.tmdbId}">
						{item.title} ({item.type === 'movie' ? 'Movie' : 'TV'})
					</option>
				{/each}
			</select>
		</label>
	</div>

	{#if loading}
		<div class="flex justify-center p-8">
			<span class="loading loading-lg loading-spinner"></span>
		</div>
	{:else if recommendations.length > 0}
		<div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
			{#each recommendations as item (item.id)}
				{#if isMovie(item)}
					<TmdbBrowseCard movie={item} />
				{:else}
					<TmdbBrowseCard tvShow={item} />
				{/if}
			{/each}
		</div>
		{#if sourceId != null && sourceType != null}
			<TmdbPagination
				{page}
				{totalPages}
				{loading}
				onpage={(p) => onload(sourceId!, sourceType!, p)}
			/>
		{/if}
	{:else if sourceId != null}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">No recommendations found for this title.</p>
		</div>
	{:else}
		<div class="rounded-lg bg-base-200 p-8 text-center">
			<p class="opacity-50">Select a title above to see recommendations.</p>
		</div>
	{/if}
{/if}
