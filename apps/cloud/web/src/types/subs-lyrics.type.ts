export interface SubsLyricsSyncedLine {
	time: number;
	text: string;
}

export type SubsLyricsKind = 'lyrics' | 'subtitle';

export interface SubsLyricsItem {
	kind: SubsLyricsKind;
	source: string;
	externalId: string;
	language?: string;
	trackName?: string;
	artistName?: string;
	albumName?: string;
	duration?: number;
	format?: string;
	url?: string;
	plainLyrics?: string;
	syncedLyrics?: SubsLyricsSyncedLine[];
	instrumental?: boolean;
	isHearingImpaired?: boolean;
	display?: string;
	sourceExternalId?: string;
}

export type SubsLyricsSearchType = 'movie' | 'tv show' | 'album';
