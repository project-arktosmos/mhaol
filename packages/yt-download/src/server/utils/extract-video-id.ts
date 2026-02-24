/** Extract video ID from a YouTube URL */
export function extractVideoId(url: string): string | null {
	const patterns = [
		/(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/embed\/|youtube\.com\/v\/)([^&\n?#]+)/,
		/youtube\.com\/shorts\/([^&\n?#]+)/
	];

	for (const pattern of patterns) {
		const match = url.match(pattern);
		if (match) return match[1];
	}
	return null;
}

/** Extract playlist ID from a YouTube URL */
export function extractPlaylistId(url: string): string | null {
	const match = url.match(/[?&]list=([^&\n]+)/);
	return match ? match[1] : null;
}

/** Check if a URL is a YouTube playlist URL */
export function isPlaylistUrl(url: string): boolean {
	return url.includes('list=') || url.includes('/playlist');
}
