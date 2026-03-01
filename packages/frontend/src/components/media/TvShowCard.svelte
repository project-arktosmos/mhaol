<script lang="ts">
	import MediaCardBase from './MediaCardBase.svelte';
	import type { MediaItem } from '$types/media-card.type';
	import type { DisplayTMDBTvShowDetails } from 'tmdb/types';

	interface Props {
		item: MediaItem;
		metadata?: DisplayTMDBTvShowDetails | null;
		loading?: boolean;
		selected?: boolean;
		onselect?: (item: MediaItem) => void;
	}

	let { item, metadata = null, loading = false, selected = false, onselect }: Props = $props();
</script>

<MediaCardBase
	{item}
	imageUrl={metadata?.posterUrl ?? null}
	imageAlt={metadata?.name ?? item.name}
	{loading}
	{selected}
	onclick={() => onselect?.(item)}
>
	{#if metadata}
		<div class="flex flex-wrap items-center gap-1 text-xs">
			<span class="font-semibold">{metadata.name}</span>
			<span class="opacity-60">
				{metadata.firstAirYear}{metadata.lastAirYear ? ` - ${metadata.lastAirYear}` : ''}
			</span>
			{#if metadata.voteAverage > 0}
				<span class="flex items-center gap-0.5">
					<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="h-3 w-3 text-yellow-500">
						<path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.007 5.404.433c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.433 2.082-5.006z" clip-rule="evenodd" />
					</svg>
					<span class="font-semibold">{metadata.voteAverage.toFixed(1)}</span>
				</span>
			{/if}
		</div>
		{#if metadata.status}
			<span class="badge badge-outline badge-xs">{metadata.status}</span>
		{/if}
		{#if metadata.genres.length > 0}
			<div class="flex flex-wrap gap-1">
				{#each metadata.genres.slice(0, 3) as genre}
					<span class="badge badge-primary badge-outline badge-xs">{genre}</span>
				{/each}
			</div>
		{/if}
		{#if metadata.numberOfSeasons || metadata.numberOfEpisodes}
			<div class="flex gap-2 text-xs opacity-70">
				{#if metadata.numberOfSeasons}
					<span>{metadata.numberOfSeasons} season{metadata.numberOfSeasons !== 1 ? 's' : ''}</span>
				{/if}
				{#if metadata.numberOfEpisodes}
					<span>{metadata.numberOfEpisodes} ep{metadata.numberOfEpisodes !== 1 ? 's' : ''}</span>
				{/if}
			</div>
		{/if}
		{#if metadata.createdBy.length > 0}
			<p class="text-xs opacity-70">{metadata.createdBy.join(', ')}</p>
		{/if}
		{#if metadata.overview}
			<p class="line-clamp-2 text-xs opacity-60">{metadata.overview}</p>
		{/if}
	{/if}
</MediaCardBase>
