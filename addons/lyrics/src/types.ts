/**
 * Lyrics types for LRCLIB API integration
 * API documentation: https://lrclib.net/docs
 */

/**
 * Response from LRCLIB API /get or /search endpoints
 */
export interface LrcLibResponse {
	id: number;
	trackName: string;
	artistName: string;
	albumName: string;
	duration: number;
	instrumental: boolean;
	plainLyrics: string | null;
	syncedLyrics: string | null;
}

/**
 * A single line of synced lyrics with timestamp
 */
export interface SyncedLyricLine {
	time: number;
	text: string;
}

/**
 * Parsed lyrics with both plain and synced versions
 */
export interface Lyrics {
	id: number;
	trackName: string;
	artistName: string;
	albumName: string;
	duration: number;
	instrumental: boolean;
	plainLyrics: string | null;
	syncedLyrics: SyncedLyricLine[] | null;
}

/**
 * Lyrics fetch status
 */
export type LyricsFetchStatus = 'idle' | 'loading' | 'success' | 'not_found' | 'error';

/**
 * Lyrics service state
 */
export interface LyricsState {
	status: LyricsFetchStatus;
	lyrics: Lyrics | null;
	error: string | null;
	currentTrackId: string | null;
}
