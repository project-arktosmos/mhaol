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
	/// Release / file name extracted from the upstream Content-Disposition
	/// header by the backend (e.g.
	/// `Captain.America.Civil.WAR.2016.1080p.HD.TC.AC3.x264-ETRG.srt`).
	release?: string;
}

export type SubsLyricsSearchType = 'movie' | 'tv show' | 'album';
