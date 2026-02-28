import { getWatchUrl } from './embed.js';

export interface YouTubeOEmbedResponse {
	title: string;
	author_name: string;
	author_url: string;
	type: string;
	height: number;
	width: number;
	version: string;
	provider_name: string;
	provider_url: string;
	thumbnail_height: number;
	thumbnail_width: number;
	thumbnail_url: string;
	html: string;
}

const OEMBED_BASE = 'https://www.youtube.com/oembed';

export async function fetchOEmbed(videoId: string): Promise<YouTubeOEmbedResponse> {
	const watchUrl = getWatchUrl(videoId);
	const url = `${OEMBED_BASE}?url=${encodeURIComponent(watchUrl)}&format=json`;

	const response = await fetch(url);
	if (!response.ok) {
		throw new Error(`YouTube oEmbed request failed: ${response.status}`);
	}

	return response.json() as Promise<YouTubeOEmbedResponse>;
}
