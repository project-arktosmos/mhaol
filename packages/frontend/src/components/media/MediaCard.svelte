<script lang="ts">
	import type { MediaItem } from '$types/media-card.type';
	import type { ImageTag } from '$types/image-tagger.type';
	import type { DisplayTMDBMovieDetails, DisplayTMDBTvShowDetails } from 'tmdb/types';
	import type { YouTubeOEmbedResponse } from 'youtube/oembed';
	import type { DisplayMusicBrainzRecording } from 'musicbrainz/types';
	import MovieCard from './MovieCard.svelte';
	import TvShowCard from './TvShowCard.svelte';
	import YouTubeCard from './YouTubeCard.svelte';
	import AudioUncategorizedCard from './AudioUncategorizedCard.svelte';
	import ImageUncategorizedCard from './ImageUncategorizedCard.svelte';
	import VideoUncategorizedCard from './VideoUncategorizedCard.svelte';

	interface Props {
		item: MediaItem;
		tmdbMetadata?: DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails | null;
		youtubeMetadata?: YouTubeOEmbedResponse | null;
		musicbrainzMetadata?: DisplayMusicBrainzRecording | null;
		metadataLoading?: boolean;
		imageTags?: ImageTag[];
		onlink?: (item: MediaItem, service: string) => void;
		onunlink?: (item: MediaItem, service: string) => void;
		onplay?: (item: MediaItem) => void;
	}

	let { item, tmdbMetadata = null, youtubeMetadata = null, musicbrainzMetadata = null, metadataLoading = false, imageTags = [], onlink, onunlink, onplay }: Props = $props();

	let cardType = $derived.by(() => {
		if (item.categoryId === 'movies' && item.links.tmdb) return 'movie';
		if (item.categoryId === 'tv' && item.links.tmdb) return 'tv';
		if (item.links.youtube) return 'youtube';
		if (item.mediaTypeId === 'audio') return 'audio';
		if (item.mediaTypeId === 'image') return 'image';
		return 'video';
	});
</script>

{#if cardType === 'movie'}
	<MovieCard {item} metadata={tmdbMetadata as DisplayTMDBMovieDetails | null} loading={metadataLoading} {onplay} {onunlink} />
{:else if cardType === 'tv'}
	<TvShowCard {item} metadata={tmdbMetadata as DisplayTMDBTvShowDetails | null} loading={metadataLoading} {onplay} {onunlink} />
{:else if cardType === 'youtube'}
	<YouTubeCard {item} metadata={youtubeMetadata} loading={metadataLoading} {onplay} {onunlink} />
{:else if cardType === 'audio'}
	<AudioUncategorizedCard {item} metadata={musicbrainzMetadata} loading={metadataLoading} {onlink} {onunlink} />
{:else if cardType === 'image'}
	<ImageUncategorizedCard {item} tags={imageTags} />
{:else}
	<VideoUncategorizedCard {item} {onplay} {onlink} />
{/if}
