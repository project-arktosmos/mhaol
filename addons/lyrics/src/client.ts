import type { LrcLibResponse, SyncedLyricLine, Lyrics } from './types.js';

const LRCLIB_API_BASE = 'https://lrclib.net/api';
const USER_AGENT = 'Mhaol/1.0.0 (https://github.com/mhaol)';

/**
 * Fetch lyrics for a track from LRCLIB API.
 * Returns the parsed Lyrics object, or null if not found.
 * Throws on network/API errors.
 */
export async function fetchLrcLibLyrics(
	trackName: string,
	artistName: string,
	albumName?: string | null,
	durationSecs?: number | null
): Promise<{ lyrics: Lyrics; raw: LrcLibResponse; status: 'success' } | { lyrics: null; raw: null; status: 'not_found' }> {
	const params = new URLSearchParams();
	params.set('track_name', trackName);
	params.set('artist_name', artistName);
	if (albumName) params.set('album_name', albumName);
	if (durationSecs && durationSecs > 0)
		params.set('duration', Math.round(durationSecs).toString());

	const response = await fetch(`${LRCLIB_API_BASE}/get?${params.toString()}`, {
		headers: { 'Lrclib-Client': USER_AGENT }
	});

	if (response.status === 404) {
		return { lyrics: null, raw: null, status: 'not_found' };
	}

	if (!response.ok) {
		throw new Error(`LRCLIB API error: ${response.status}`);
	}

	const data: LrcLibResponse = await response.json();
	return { lyrics: parseResponse(data), raw: data, status: 'success' };
}

/**
 * Search for lyrics using a text query.
 */
export async function searchLrcLibLyrics(query: string): Promise<Lyrics[]> {
	const params = new URLSearchParams({ q: query });
	const response = await fetch(`${LRCLIB_API_BASE}/search?${params.toString()}`, {
		headers: { 'Lrclib-Client': USER_AGENT }
	});

	if (!response.ok) {
		throw new Error(`LRCLIB API error: ${response.status}`);
	}

	const data: LrcLibResponse[] = await response.json();
	return data.map((item) => parseResponse(item));
}

/**
 * Parse LRCLIB response into internal Lyrics format.
 */
export function parseResponse(data: LrcLibResponse): Lyrics {
	return {
		id: data.id,
		trackName: data.trackName,
		artistName: data.artistName,
		albumName: data.albumName,
		duration: data.duration,
		instrumental: data.instrumental,
		plainLyrics: data.plainLyrics,
		syncedLyrics: data.syncedLyrics ? parseLrcToSyncedLines(data.syncedLyrics) : null
	};
}

/**
 * Parse LRC format synced lyrics into structured array.
 * LRC format: [mm:ss.xx] Line text
 */
export function parseLrcToSyncedLines(lrc: string): SyncedLyricLine[] {
	const lines: SyncedLyricLine[] = [];
	const regex = /\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)/g;
	let match;

	while ((match = regex.exec(lrc)) !== null) {
		const minutes = parseInt(match[1], 10);
		const seconds = parseInt(match[2], 10);
		const milliseconds = parseInt(match[3].padEnd(3, '0'), 10);
		const text = match[4].trim();

		const time = minutes * 60 + seconds + milliseconds / 1000;
		lines.push({ time, text });
	}

	lines.sort((a, b) => a.time - b.time);

	return lines;
}
