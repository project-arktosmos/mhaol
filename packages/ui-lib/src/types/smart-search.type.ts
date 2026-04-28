import type {
	TorrentSearchResult,
	TorrentAnalysis
} from 'addons/torrent-search-thepiratebay/types';

export type { TorrentAnalysis } from 'addons/torrent-search-thepiratebay/types';

export type SmartSearchMode = 'download' | 'stream' | 'fetch';

export type SmartSearchMediaType = 'movies' | 'tv' | 'music' | 'games' | 'books';

export interface SmartSearchMediaConfig {
	preferredLanguage?: string;
	preferredQuality?: string;
	preferredConsole?: string;
	preferredFormat?: string;
	smartSearchPrompt: string;
}

export type SmartSearchAllConfigs = Record<SmartSearchMediaType, SmartSearchMediaConfig>;

interface SmartSearchBaseSelection {
	title: string;
	year: string;
	mode: SmartSearchMode;
}

/**
 * Language hint that drives indexer selection for a search:
 *  - `en` (default): PirateBay only
 *  - `es`: PirateBay + Spanish-enriched queries + Spanish-language indexers
 */
export type SmartSearchLang = 'en' | 'es';

export interface SmartSearchMovieSelection extends SmartSearchBaseSelection {
	type: 'movie';
	tmdbId: number;
	existingItemId?: string;
	existingLibraryId?: string;
	searchLang?: SmartSearchLang;
}

export interface TvSeasonMeta {
	seasonNumber: number;
	name: string;
	episodeCount: number;
	episodes: TvEpisodeMeta[];
}

export interface TvEpisodeMeta {
	episodeNumber: number;
	seasonNumber: number;
	name: string;
}

export interface SmartSearchTvSelection extends SmartSearchBaseSelection {
	type: 'tv';
	tmdbId: number;
	existingItemId?: string;
	existingLibraryId?: string;
	seasons?: TvSeasonMeta[];
}

export interface SmartSearchMusicSelection extends SmartSearchBaseSelection {
	type: 'music';
	musicbrainzId: string;
	artist: string;
	musicSearchMode?: 'album';
}

export interface SmartSearchGameSelection extends SmartSearchBaseSelection {
	type: 'game';
	retroachievementsId: number;
	consoleName: string;
}

export interface SmartSearchBookSelection extends SmartSearchBaseSelection {
	type: 'book';
	openlibraryKey: string;
	author: string;
}

export type SmartSearchSelection =
	| SmartSearchMovieSelection
	| SmartSearchTvSelection
	| SmartSearchMusicSelection
	| SmartSearchGameSelection
	| SmartSearchBookSelection;

export interface SmartSearchTorrentResult extends TorrentSearchResult {
	searchQueries: string[];
	analysis: TorrentAnalysis | null;
	analyzing: boolean;
}

export type TvTorrentScope = 'complete' | 'season' | 'episode';

export type MusicTorrentScope = 'album' | 'discography';

export interface MusicAlbumMeta {
	id: string;
	title: string;
	year: string;
}

export interface MusicSmartSearchResults {
	album: SmartSearchTorrentResult[];
	discography: SmartSearchTorrentResult[];
}

export interface TvSeasonResults {
	seasonPacks: SmartSearchTorrentResult[];
	episodes: Record<number, SmartSearchTorrentResult[]>;
}

export interface TvSmartSearchResults {
	complete: SmartSearchTorrentResult[];
	seasons: Record<number, TvSeasonResults>;
}

export interface TvFetchedCandidates {
	complete: SmartSearchTorrentResult | null;
	seasons: Record<number, SmartSearchTorrentResult | null>;
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
	fetchedTvCandidates: TvFetchedCandidates | null;
	tvResults: TvSmartSearchResults | null;
	tvSeasonsMeta: TvSeasonMeta[] | null;
	activeTvTab: 'complete' | number;
	musicResults: MusicSmartSearchResults | null;
	activeMusicTab: 'album' | 'discography';
}
