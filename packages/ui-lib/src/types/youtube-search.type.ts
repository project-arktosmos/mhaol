export interface YouTubeSearchItem {
	videoId: string;
	type: string;
	url: string;
	title: string;
	thumbnail: string;
	duration: number;
	durationText: string;
	views: number;
	viewsText: string;
	uploadedDate: string;
	uploaderName: string;
	uploaderUrl: string;
	uploaderAvatar: string;
	uploaderVerified: boolean;
}

export interface YouTubeSearchChannelItem {
	type: string;
	channelId: string;
	name: string;
	thumbnail: string;
	url: string;
	subscriberText: string;
	videoCountText: string;
	description: string;
	verified: boolean;
}

export interface YouTubeSearchResponse {
	items: YouTubeSearchItem[];
	channels: YouTubeSearchChannelItem[];
	continuation: string | null;
}

export interface YouTubeSearchState {
	query: string;
	searching: boolean;
	results: YouTubeSearchItem[];
	channels: YouTubeSearchChannelItem[];
	continuation: string | null;
	loadingMore: boolean;
	error: string | null;
}
