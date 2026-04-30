export interface YouTubeSearchItem {
	videoId: string;
	url: string;
	title: string;
	uploaderName: string;
	duration: number;
}

interface YouTubeSearchResponse {
	items?: YouTubeSearchItem[];
}

export async function searchYouTube(query: string): Promise<YouTubeSearchItem[]> {
	const res = await fetch(`/api/ytdl/search?q=${encodeURIComponent(query)}`);
	if (!res.ok) throw new Error(`HTTP ${res.status}`);
	const data = (await res.json()) as YouTubeSearchResponse;
	return data.items ?? [];
}

const NOISE_WORDS =
	/\b(official|video|audio|lyrics?|hd|4k|mv|hq|live|remaster(?:ed)?|edit|version)\b/g;

function normalize(s: string): string {
	return s
		.toLowerCase()
		.replace(/[\(\[][^\)\]]*[\)\]]/g, ' ')
		.replace(NOISE_WORDS, ' ')
		.replace(/[^a-z0-9]+/g, ' ')
		.trim();
}

function tokens(s: string): string[] {
	return normalize(s)
		.split(' ')
		.filter((w) => w.length > 1);
}

export function pickBestYouTubeMatch(
	items: YouTubeSearchItem[],
	trackTitle: string,
	artist: string,
	albumTitle: string,
	trackDurationMs: number | null
): YouTubeSearchItem | null {
	if (items.length === 0) return null;
	const trackTokens = tokens(trackTitle);
	const artistTokens = tokens(artist);
	const albumTokens = tokens(albumTitle);
	const targetSec =
		trackDurationMs && trackDurationMs > 0 ? Math.round(trackDurationMs / 1000) : null;

	let best: { item: YouTubeSearchItem; score: number } | null = null;
	for (const item of items) {
		const titleNorm = normalize(item.title);
		const uploaderNorm = normalize(item.uploaderName);
		const titleAndUploader = `${titleNorm} ${uploaderNorm}`;

		const trackHits = trackTokens.filter((t) => titleNorm.includes(t)).length;
		const trackRatio = trackTokens.length > 0 ? trackHits / trackTokens.length : 0;
		if (trackRatio < 0.5) continue;

		let score = trackRatio * 10;
		if (artistTokens.length > 0) {
			const artistHits = artistTokens.filter((t) => titleAndUploader.includes(t)).length;
			score += (artistHits / artistTokens.length) * 6;
		}
		if (albumTokens.length > 0) {
			const albumHits = albumTokens.filter((t) => titleNorm.includes(t)).length;
			score += (albumHits / albumTokens.length) * 2;
		}
		if (targetSec && item.duration > 0) {
			const delta = Math.abs(item.duration - targetSec);
			if (delta <= 3) score += 6;
			else if (delta <= 10) score += 3;
			else if (delta <= 20) score += 1;
		}
		if (!best || score > best.score) best = { item, score };
	}
	return best?.item ?? null;
}

export async function resolveYouTubeUrlForTrack(
	trackTitle: string,
	artist: string,
	albumTitle: string,
	trackDurationMs: number | null
): Promise<string | null> {
	const parts = [artist, albumTitle, trackTitle].map((s) => s.trim()).filter(Boolean);
	if (parts.length === 0) return null;
	const query = parts.join(' ');
	const items = await searchYouTube(query);
	const match = pickBestYouTubeMatch(items, trackTitle, artist, albumTitle, trackDurationMs);
	return match?.url ?? null;
}
