declare module 'youtube/oembed' {
	export interface YouTubeOEmbedResponse {
		title: string;
		author_name: string;
		thumbnail_url: string;
		html: string;
	}
}

declare module 'youtube/embed' {
	export function getThumbnailUrl(videoId: string): string;
}
