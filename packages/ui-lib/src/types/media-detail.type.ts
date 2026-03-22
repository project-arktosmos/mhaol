import type { MediaItem } from 'ui-lib/types/media-card.type';
import type { ImageTag } from 'ui-lib/types/image-tagger.type';
import type { DisplayTMDBMovieDetails, DisplayTMDBTvShowDetails } from 'addons/tmdb/types';
import type { YouTubeOEmbedResponse } from 'ui-lib/types/youtube-oembed.type';
import type { DisplayMusicBrainzRecording } from 'ui-lib/types/musicbrainz.type';

export type MediaDetailCardType = 'movie' | 'tv' | 'youtube' | 'audio' | 'image' | 'video';

export interface MediaDetailSelection {
	item: MediaItem;
	cardType: MediaDetailCardType;
	tmdbMetadata: DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails | null;
	youtubeMetadata: YouTubeOEmbedResponse | null;
	musicbrainzMetadata: DisplayMusicBrainzRecording | null;
	imageTags: ImageTag[];
	imageTagging?: boolean;
	onplay?: (item: MediaItem) => void;
	onlink?: (item: MediaItem, service: string) => void;
	onunlink?: (item: MediaItem, service: string) => void;
	ontagimage?: (item: MediaItem) => void;
	onaddtag?: (item: MediaItem, tag: string) => void;
	onremovetag?: (item: MediaItem, tag: string) => void;
}
