export interface YouTubeSearchItem {
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

export interface YouTubeSearchResponse {
	items: YouTubeSearchItem[];
	continuation: string | null;
}

export interface YouTubeSearchState {
	query: string;
	searching: boolean;
	results: YouTubeSearchItem[];
	continuation: string | null;
	loadingMore: boolean;
	error: string | null;
}
