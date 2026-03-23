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
