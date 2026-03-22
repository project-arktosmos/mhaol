export interface SyncedLine {
	time: number;
	text: string;
}

export interface Lyrics {
	id: string;
	artist: string;
	title: string;
	lines: string[];
	plainLyrics?: string;
	syncedLyrics?: SyncedLine[];
	instrumental?: boolean;
}

export interface LyricsState {
	status: 'idle' | 'loading' | 'success' | 'not_found' | 'error' | 'done';
	loading?: boolean;
	error: string | null;
	lyrics: Lyrics | null;
	currentTrackId: string | null;
}
