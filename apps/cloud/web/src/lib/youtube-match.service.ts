import type { YouTubeStreamFormat, YouTubeStreamUrlResult } from 'addons/youtube/types';
import { extractVideoId } from 'addons/youtube/types';
import { playerService } from '$services/player.service';
import type { PlayableFile } from '$types/player.type';

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
	if (!match) return null;
	return match.videoId ? `https://www.youtube.com/watch?v=${match.videoId}` : null;
}

// Words that strongly imply a result is *not* a trailer — reactions,
// recaps, fan-edits, or commentary clips that share most of the title's
// tokens with the actual trailer and would otherwise score highly.
const TRAILER_NEGATIVE =
	/\b(reaction|review|recap|breakdown|explained|analysis|fanmade|fan made|behind the scenes|making of|interview|spoilers?)\b/;

/**
 * Pick the best trailer match from a YouTube search response. Mirrors the
 * music-track double-dip:
 *
 *   1. **First dip** — require ≥50% of the *title tokens* (movie title or
 *      show title) to appear in the result's title. Filters out clips
 *      that only share generic words like "trailer".
 *   2. **Second dip** — score across multiple dimensions: title overlap,
 *      whether the result mentions "trailer", whether the year appears,
 *      and (for TV) whether the season tag (`s01`, `season 1`, …) is
 *      present. Negative keywords (reactions, reviews, recaps) penalise
 *      the score so commentary clips lose to the actual trailer even
 *      when the title is similar.
 *
 * Returns `null` when nothing crosses the threshold.
 */
export function pickBestTrailerMatch(
	items: YouTubeSearchItem[],
	itemTitle: string,
	year: number | null,
	seasonNumber: number | null
): YouTubeSearchItem | null {
	if (items.length === 0) return null;
	const titleTokens = tokens(itemTitle);
	const yearStr = year && Number.isFinite(year) ? String(year) : null;
	const seasonTags =
		seasonNumber !== null && Number.isFinite(seasonNumber) && seasonNumber > 0
			? [`season ${seasonNumber}`, `s${String(seasonNumber).padStart(2, '0')}`, `s${seasonNumber}`]
			: [];

	let best: { item: YouTubeSearchItem; score: number } | null = null;
	for (const item of items) {
		const titleNorm = normalize(item.title);
		const uploaderNorm = normalize(item.uploaderName);
		const titleAndUploader = `${titleNorm} ${uploaderNorm}`;

		const titleHits = titleTokens.filter((t) => titleNorm.includes(t)).length;
		const titleRatio = titleTokens.length > 0 ? titleHits / titleTokens.length : 0;
		if (titleRatio < 0.5) continue;
		// "trailer" must appear somewhere — same gating step as the music
		// flow's track-title check, just with a different required token.
		if (!titleAndUploader.includes('trailer')) continue;

		// For per-season TV trailers, the season tag is required: a generic
		// show trailer that doesn't name the season is the wrong match.
		if (seasonTags.length > 0) {
			const hasSeasonTag = seasonTags.some((tag) => titleNorm.includes(tag));
			if (!hasSeasonTag) continue;
		}

		let score = titleRatio * 10;
		if (titleNorm.includes('trailer')) score += 5;
		if (yearStr && titleAndUploader.includes(yearStr)) score += 3;
		if (seasonTags.length > 0) {
			const tagHits = seasonTags.filter((t) => titleNorm.includes(t)).length;
			score += tagHits * 2;
		}
		if (TRAILER_NEGATIVE.test(titleNorm)) score -= 12;
		if (TRAILER_NEGATIVE.test(uploaderNorm)) score -= 6;

		if (!best || score > best.score) best = { item, score };
	}
	return best?.item ?? null;
}

export async function resolveYouTubeTrailerForMovie(
	title: string,
	year: number | null
): Promise<string | null> {
	const trimmed = title.trim();
	if (!trimmed) return null;
	const yearPart = year && Number.isFinite(year) ? String(year) : '';
	const query = [trimmed, yearPart, 'trailer'].filter(Boolean).join(' ');
	const items = await searchYouTube(query);
	const match = pickBestTrailerMatch(items, trimmed, year, null);
	if (!match) return null;
	return match.videoId ? `https://www.youtube.com/watch?v=${match.videoId}` : null;
}

export async function resolveYouTubeTrailerForSeason(
	showTitle: string,
	seasonNumber: number,
	airYear: number | null
): Promise<string | null> {
	const trimmed = showTitle.trim();
	if (!trimmed || !Number.isFinite(seasonNumber) || seasonNumber <= 0) return null;
	const query = `${trimmed} season ${seasonNumber} trailer`;
	const items = await searchYouTube(query);
	const match = pickBestTrailerMatch(items, trimmed, airYear, seasonNumber);
	if (!match) return null;
	return match.videoId ? `https://www.youtube.com/watch?v=${match.videoId}` : null;
}

function pickAudioFormat(result: YouTubeStreamUrlResult): YouTubeStreamFormat | null {
	const muxed = result.formats.filter((f) => !f.isAudioOnly && !f.isVideoOnly);
	if (muxed.length > 0) {
		muxed.sort((a, b) => {
			const heightDiff = (b.height ?? 0) - (a.height ?? 0);
			if (heightDiff !== 0) return heightDiff;
			return b.bitrate - a.bitrate;
		});
		return muxed[0];
	}
	const audioOnly = result.formats.filter((f) => f.isAudioOnly);
	const mp4Audio = audioOnly.filter((f) => f.container === 'mp4');
	const sortedByBitrate = (list: YouTubeStreamFormat[]) =>
		[...list].sort((a, b) => b.bitrate - a.bitrate);
	return sortedByBitrate(mp4Audio)[0] ?? sortedByBitrate(audioOnly)[0] ?? null;
}

export async function playYouTubeAudio(
	youtubeUrl: string,
	title: string,
	thumbnailUrl: string | null = null,
	durationSeconds: number | null = null
): Promise<void> {
	const res = await fetch(
		`/api/ytdl/info/stream-urls-browser?url=${encodeURIComponent(youtubeUrl)}`
	);
	if (!res.ok) {
		const body = await res.text();
		throw new Error(body || `HTTP ${res.status}`);
	}
	const result = (await res.json()) as YouTubeStreamUrlResult;
	const format = pickAudioFormat(result);
	if (!format) throw new Error('No playable audio format');
	const videoId = extractVideoId(youtubeUrl) ?? youtubeUrl;
	const file: PlayableFile = {
		id: `youtube:${videoId}:audio`,
		type: 'youtube',
		name: title,
		outputPath: '',
		mode: 'audio',
		format: null,
		videoFormat: null,
		thumbnailUrl,
		durationSeconds,
		size: format.contentLength ?? 0,
		completedAt: ''
	};
	await playerService.playUrl(file, format.url, format.mimeType, 'sidebar');
}

export async function playYouTubeVideo(
	youtubeUrl: string,
	title: string,
	thumbnailUrl: string | null = null,
	durationSeconds: number | null = null
): Promise<void> {
	const res = await fetch(
		`/api/ytdl/info/stream-urls-browser?url=${encodeURIComponent(youtubeUrl)}`
	);
	if (!res.ok) {
		const body = await res.text();
		throw new Error(body || `HTTP ${res.status}`);
	}
	const result = (await res.json()) as YouTubeStreamUrlResult;
	// Same picker as audio: prefers muxed (video+audio) when available so
	// the right-side `<video>` element gets a single playable URL. Falling
	// back to audio-only is fine for the rare case where YouTube only
	// exposes split streams in the browser-safe set.
	const format = pickAudioFormat(result);
	if (!format) throw new Error('No playable trailer format');
	const videoId = extractVideoId(youtubeUrl) ?? youtubeUrl;
	const file: PlayableFile = {
		id: `youtube:${videoId}:video`,
		type: 'youtube',
		name: title,
		outputPath: '',
		mode: 'video',
		format: null,
		videoFormat: null,
		thumbnailUrl,
		durationSeconds,
		size: format.contentLength ?? 0,
		completedAt: ''
	};
	await playerService.playUrl(file, format.url, format.mimeType, 'sidebar');
}
