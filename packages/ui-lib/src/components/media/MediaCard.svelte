<script lang="ts">
	import type { MediaItem } from 'ui-lib/types/media-card.type';
	import type { ImageTag } from 'ui-lib/types/image-tagger.type';
	import type { TorrentState } from 'ui-lib/types/torrent.type';
	import type { DisplayTMDBMovieDetails, DisplayTMDBTvShowDetails } from 'addons/tmdb/types';
	import type { YouTubeOEmbedResponse } from 'addons/youtube/types';
	import type { DisplayMusicBrainzRecording } from 'addons/musicbrainz/types';
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
		tagging?: boolean;
		selected?: boolean;
		torrentProgress?: number | null;
		torrentState?: TorrentState | null;
		torrentSpeed?: number | null;
		torrentEta?: number | null;
		onselect?: (item: MediaItem) => void;
	}

	let {
		item,
		tmdbMetadata = null,
		youtubeMetadata = null,
		musicbrainzMetadata = null,
		metadataLoading = false,
		imageTags = [],
		tagging = false,
		selected = false,
		torrentProgress = null,
		torrentState = null,
		torrentSpeed = null,
		torrentEta = null,
		onselect
	}: Props = $props();

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
	<MovieCard
		{item}
		metadata={tmdbMetadata as DisplayTMDBMovieDetails | null}
		loading={metadataLoading}
		{onselect}
		{selected}
		{torrentProgress}
		{torrentState}
		{torrentSpeed}
		{torrentEta}
	/>
{:else if cardType === 'tv'}
	<TvShowCard
		{item}
		metadata={tmdbMetadata as DisplayTMDBTvShowDetails | null}
		loading={metadataLoading}
		{onselect}
		{selected}
		{torrentProgress}
		{torrentState}
		{torrentSpeed}
		{torrentEta}
	/>
{:else if cardType === 'youtube'}
	<YouTubeCard {item} metadata={youtubeMetadata} loading={metadataLoading} {onselect} {selected} />
{:else if cardType === 'audio'}
	<AudioUncategorizedCard
		{item}
		metadata={musicbrainzMetadata}
		loading={metadataLoading}
		{onselect}
		{selected}
	/>
{:else if cardType === 'image'}
	<ImageUncategorizedCard {item} tags={imageTags} {tagging} {onselect} {selected} />
{:else}
	<VideoUncategorizedCard {item} {onselect} {selected} />
{/if}
