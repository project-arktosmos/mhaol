<script lang="ts">
	import type { MediaItem } from '$types/media-card.type';
	import type { DisplayTMDBMovieDetails, DisplayTMDBTvShowDetails } from 'addons/tmdb/types';
	import MovieCard from './MovieCard.svelte';
	import TvShowCard from './TvShowCard.svelte';
	import VideoUncategorizedCard from './VideoUncategorizedCard.svelte';

	interface Props {
		item: MediaItem;
		tmdbMetadata?: DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails | null;
		metadataLoading?: boolean;
		selected?: boolean;
		onselect?: (item: MediaItem) => void;
	}

	let {
		item,
		tmdbMetadata = null,
		metadataLoading = false,
		selected = false,
		onselect
	}: Props = $props();

	let cardType = $derived.by(() => {
		if (item.categoryId === 'movies' && item.links.tmdb) return 'movie';
		if (item.categoryId === 'tv' && item.links.tmdb) return 'tv';
		return 'video';
	});
</script>

{#if cardType === 'movie'}
	<MovieCard
		{item}
		metadata={tmdbMetadata as DisplayTMDBMovieDetails | null}
		loading={metadataLoading}
		{onselect}
		{selected}
	/>
{:else if cardType === 'tv'}
	<TvShowCard
		{item}
		metadata={tmdbMetadata as DisplayTMDBTvShowDetails | null}
		loading={metadataLoading}
		{onselect}
		{selected}
	/>
{:else}
	<VideoUncategorizedCard {item} {onselect} {selected} />
{/if}
