import type { TorrentSearchResult } from 'addons/torrent-search-thepiratebay/types';

export interface SmartSearchSelection {
	title: string;
	year: string;
	type: 'movie' | 'tv';
}

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
}
