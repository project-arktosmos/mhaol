// Re-export API types from the lrclib addon
export type { SyncedLine, Lyrics } from 'addons/lrclib/types';

import type { Lyrics } from 'addons/lrclib/types';

export interface LyricsState {
	status: 'idle' | 'loading' | 'success' | 'not_found' | 'error' | 'done';
	loading?: boolean;
	error: string | null;
	lyrics: Lyrics | null;
	currentTrackId: string | null;
}
