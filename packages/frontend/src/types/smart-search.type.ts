import type { TorrentSearchResult } from 'addons/torrent-search-thepiratebay/types';

export type SmartSearchMode = 'download' | 'stream' | 'fetch';

interface SmartSearchBaseSelection {
	title: string;
	year: string;
	mode: SmartSearchMode;
}

export interface SmartSearchMovieSelection extends SmartSearchBaseSelection {
	type: 'movie' | 'tv';
	tmdbId: number;
}

export interface SmartSearchMusicSelection extends SmartSearchBaseSelection {
	type: 'music';
	musicbrainzId: string;
	artist: string;
}

export type SmartSearchSelection = SmartSearchMovieSelection | SmartSearchMusicSelection;

export interface TorrentAnalysis {
	quality: string;
	languages: string;
	subs: string;
	relevance: number;
	reason: string;
}

export interface SmartSearchTorrentResult extends TorrentSearchResult {
	searchQueries: string[];
	analysis: TorrentAnalysis | null;
	analyzing: boolean;
}

export interface SmartSearchState {
	selection: SmartSearchSelection | null;
	visible: boolean;
	searching: boolean;
	analyzing: boolean;
	searchResults: SmartSearchTorrentResult[];
	searchError: string | null;
	streamingHash: string | null;
	streamingProgress: number;
	pendingItemId: string | null;
	pendingLibraryId: string | null;
	downloadedHash: string | null;
	fetchedCandidate: SmartSearchTorrentResult | null;
}
