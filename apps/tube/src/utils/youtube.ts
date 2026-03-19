const YOUTUBE_THUMBNAIL_BASE = 'https://img.youtube.com/vi';

export function getThumbnailUrl(
	videoId: string,
	quality: 'default' | 'mqdefault' | 'hqdefault' | 'sddefault' | 'maxresdefault' = 'hqdefault'
): string {
	return `${YOUTUBE_THUMBNAIL_BASE}/${videoId}/${quality}.jpg`;
}
