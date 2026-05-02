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

export type DownloadStatus =
	| 'idle'
	| 'pending'
	| 'downloading'
	| 'completed'
	| 'failed'
	| 'skipped';

export interface TrackEntry {
	id: string;
	position: number;
	title: string;
	lengthMs: number | null;
	youtubeUrl: string | null;
	youtubeStatus: EntryStatus;
	lyricsStatus: EntryStatus;
	lyrics: SubsLyricsItem | null;
	/// CID of the locally-downloaded audio file when an `ipfs`-typed
	/// FileEntry exists for this track on the firkin. Drives the per-row
	/// "Play" button (vs the YouTube-streaming "Stream" button).
	localCid: string | null;
	/// Live status from the album-download background task, projected from
	/// `/api/firkins/:id/download-progress` while the task is running.
	downloadStatus: DownloadStatus;
	downloadProgress: number;
	downloadError: string | null;
}

export type PersistTrailers = (resolved: Trailer[]) => Promise<void>;
