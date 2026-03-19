import type { MediaItem } from '$types/media-card.type';
import type { DisplayTMDBMovieDetails, DisplayTMDBTvShowDetails } from 'tmdb/types';

export type MediaDetailCardType = 'movie' | 'tv' | 'video';

export interface MediaDetailSelection {
	item: MediaItem;
	cardType: MediaDetailCardType;
	tmdbMetadata: DisplayTMDBMovieDetails | DisplayTMDBTvShowDetails | null;
	onplay?: (item: MediaItem) => void;
	onlink?: (item: MediaItem, service: string) => void;
	onunlink?: (item: MediaItem, service: string) => void;
}
