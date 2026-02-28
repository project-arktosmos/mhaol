import type { YouTubeVideoInfo } from './types.js';

const YOUTUBE_EMBED_BASE = 'https://www.youtube.com/embed';
const YOUTUBE_THUMBNAIL_BASE = 'https://img.youtube.com/vi';
const YOUTUBE_WATCH_BASE = 'https://www.youtube.com/watch';

const YOUTUBE_ID_REGEX = /^[a-zA-Z0-9_-]{11}$/;
const YOUTUBE_URL_REGEX =
	/(?:youtube\.com\/(?:watch\?v=|embed\/|v\/|shorts\/)|youtu\.be\/)([a-zA-Z0-9_-]{11})/;

export function isValidYouTubeId(id: string): boolean {
	return YOUTUBE_ID_REGEX.test(id);
}

export function extractYouTubeId(input: string): string | null {
	const trimmed = input.trim();
	if (YOUTUBE_ID_REGEX.test(trimmed)) return trimmed;
	const match = trimmed.match(YOUTUBE_URL_REGEX);
	return match ? match[1] : null;
}

export function getEmbedUrl(videoId: string, autoplay = false): string {
	const params = new URLSearchParams({ rel: '0' });
	if (autoplay) params.set('autoplay', '1');
	return `${YOUTUBE_EMBED_BASE}/${videoId}?${params.toString()}`;
}

export function getThumbnailUrl(
	videoId: string,
	quality: 'default' | 'mqdefault' | 'hqdefault' | 'sddefault' | 'maxresdefault' = 'hqdefault'
): string {
	return `${YOUTUBE_THUMBNAIL_BASE}/${videoId}/${quality}.jpg`;
}

export function getWatchUrl(videoId: string): string {
	return `${YOUTUBE_WATCH_BASE}?v=${videoId}`;
}

export function getVideoInfo(videoId: string): YouTubeVideoInfo {
	return {
		videoId,
		embedUrl: getEmbedUrl(videoId),
		thumbnailUrl: getThumbnailUrl(videoId),
		watchUrl: getWatchUrl(videoId)
	};
}
