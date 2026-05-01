import { base } from '$app/paths';
import type { SubsLyricsItem } from '$types/subs-lyrics.type';

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

export function pickBestLyricsMatch(
	items: SubsLyricsItem[],
	trackTitle: string,
	artist: string,
	albumTitle: string,
	trackDurationMs: number | null
): SubsLyricsItem | null {
	if (items.length === 0) return null;
	const trackTokens = tokens(trackTitle);
	const artistTokens = tokens(artist);
	const albumTokens = tokens(albumTitle);
	const targetSec =
		trackDurationMs && trackDurationMs > 0 ? Math.round(trackDurationMs / 1000) : null;

	let best: { item: SubsLyricsItem; score: number } | null = null;
	for (const item of items) {
		if (item.kind !== 'lyrics') continue;
		const itemTrack = normalize(item.trackName ?? '');
		const itemArtist = normalize(item.artistName ?? '');
		const itemAlbum = normalize(item.albumName ?? '');

		const trackHits = trackTokens.filter((t) => itemTrack.includes(t)).length;
		const trackRatio = trackTokens.length > 0 ? trackHits / trackTokens.length : 0;
		if (trackRatio < 0.5) continue;

		let score = trackRatio * 10;
		if (artistTokens.length > 0) {
			const artistHits = artistTokens.filter((t) => itemArtist.includes(t)).length;
			score += (artistHits / artistTokens.length) * 6;
		}
		if (albumTokens.length > 0) {
			const albumHits = albumTokens.filter((t) => itemAlbum.includes(t)).length;
			score += (albumHits / albumTokens.length) * 2;
		}
		if (targetSec && item.duration && item.duration > 0) {
			const delta = Math.abs(item.duration - targetSec);
			if (delta <= 3) score += 6;
			else if (delta <= 10) score += 3;
			else if (delta <= 20) score += 1;
		}
		if ((item.syncedLyrics?.length ?? 0) > 0) score += 1;

		if (!best || score > best.score) best = { item, score };
	}
	return best?.item ?? null;
}

export async function resolveLyricsForTrack(
	trackTitle: string,
	artist: string,
	albumTitle: string,
	trackDurationMs: number | null
): Promise<SubsLyricsItem | null> {
	const parts = [artist, trackTitle].map((s) => s.trim()).filter(Boolean);
	if (parts.length === 0) return null;
	const query = parts.join(' ');
	const res = await fetch(`${base}/api/search/subs-lyrics`, {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify({ addon: 'lrclib', query })
	});
	if (!res.ok) throw new Error(`HTTP ${res.status}`);
	const items = (await res.json()) as SubsLyricsItem[];
	return pickBestLyricsMatch(items, trackTitle, artist, albumTitle, trackDurationMs);
}
