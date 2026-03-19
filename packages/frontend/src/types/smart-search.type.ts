import type { TorrentSearchResult } from 'addons/torrent-search-thepiratebay/types';

export interface SmartSearchSelection {
	title: string;
	year: string;
	type: 'movie' | 'tv';
}

export interface SmartSearchState {
	selection: SmartSearchSelection | null;
	visible: boolean;
	searching: boolean;
	searchResults: TorrentSearchResult[];
	searchError: string | null;
}
