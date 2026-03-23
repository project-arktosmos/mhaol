import type { YouTubeSearchItem, YouTubeSearchChannelItem } from 'addons/youtube/types';

// Re-export API types from the youtube addon
export type {
	YouTubeSearchItem,
	YouTubeSearchChannelItem,
	YouTubeSearchResponse
} from 'addons/youtube/types';

export interface YouTubeSearchState {
	query: string;
	searching: boolean;
	results: YouTubeSearchItem[];
	channels: YouTubeSearchChannelItem[];
	continuation: string | null;
	loadingMore: boolean;
	error: string | null;
}
