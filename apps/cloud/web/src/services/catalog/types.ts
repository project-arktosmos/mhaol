import type { Trailer } from '$lib/firkins.service';
import type { SubsLyricsItem } from '$types/subs-lyrics.type';

export type ResolutionStatus = 'idle' | 'loading' | 'done' | 'error';
export type EntryStatus = 'idle' | 'pending' | 'searching' | 'found' | 'missing' | 'error';

export interface TrailerEntry {
	key: string;
	label: string | null;
	seasonNumber: number | null;
	airYear: number | null;
	youtubeUrl: string | null;
	language: string | null;
	status: EntryStatus;
}

export interface TrackEntry {
	id: string;
	position: number;
	title: string;
	lengthMs: number | null;
	youtubeUrl: string | null;
	youtubeStatus: EntryStatus;
	lyricsStatus: EntryStatus;
	lyrics: SubsLyricsItem | null;
}

export type PersistTrailers = (resolved: Trailer[]) => Promise<void>;
