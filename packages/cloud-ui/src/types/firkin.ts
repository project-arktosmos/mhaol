/**
 * Shared firkin types used across the cloud WebUI and the browser-side
 * player app. Kept identical to the cloud server's serialized shape so a
 * firkin body fetched from any source — cloud HTTP API or directly via
 * IPFS UnixFS — can be passed to the same Svelte components without
 * conversion.
 */

export const FIRKIN_ADDONS = [
	'tmdb-movie',
	'tmdb-tv',
	'musicbrainz',
	'youtube-video',
	'wyzie-subs-movie',
	'wyzie-subs-tv',
	'lrclib',
	'local-movie',
	'local-tv',
	'local-album',
	'local-book',
	'local-game'
] as const;

export type FirkinAddon = (typeof FIRKIN_ADDONS)[number];

export const FIRKIN_KINDS = [
	'movie',
	'tv show',
	'album',
	'youtube video',
	'book',
	'game'
] as const;

export type FirkinKind = (typeof FIRKIN_KINDS)[number];

export const ADDON_KIND: Record<FirkinAddon, FirkinKind> = {
	'tmdb-movie': 'movie',
	'tmdb-tv': 'tv show',
	musicbrainz: 'album',
	'youtube-video': 'youtube video',
	'wyzie-subs-movie': 'movie',
	'wyzie-subs-tv': 'tv show',
	lrclib: 'album',
	'local-movie': 'movie',
	'local-tv': 'tv show',
	'local-album': 'album',
	'local-book': 'book',
	'local-game': 'game'
};

export function addonKind(addon: string): FirkinKind | null {
	return (addon as FirkinAddon) in ADDON_KIND ? ADDON_KIND[addon as FirkinAddon] : null;
}

export interface FirkinArtist {
	id?: string;
	name: string;
	role?: string;
	roles?: string[];
	imageUrl?: string;
}

export interface FirkinImage {
	url: string;
	mimeType: string;
	fileSize: number;
	width: number;
	height: number;
}

export const FIRKIN_FILE_TYPES = ['ipfs', 'torrent magnet', 'url', 'lyrics'] as const;
export type FirkinFileType = (typeof FIRKIN_FILE_TYPES)[number];

export interface FirkinFile {
	type: FirkinFileType;
	value: string;
	title?: string;
}

export interface FirkinTrailer {
	youtubeUrl: string;
	label?: string;
	language?: string;
}

/**
 * One user-rating snapshot attached to a firkin, sourced from an upstream
 * catalog API (TMDB `vote_average`/`vote_count`, MusicBrainz
 * `rating.value`/`votes-count`). `score` is the raw upstream value;
 * `maxScore` is its scale (TMDB = 10, MusicBrainz = 5) so the UI can
 * render `score / maxScore` without per-source rules.
 */
export interface FirkinReview {
	label: string;
	score: number;
	maxScore: number;
	voteCount?: number;
}

export interface Firkin {
	/** Stable UUID record id — assigned at create time and never changed. */
	id: string;
	/** Content-addressed CID of the body, recomputed on every update. */
	cid: string;
	title: string;
	artistIds?: string[];
	artists: FirkinArtist[];
	description: string;
	images: FirkinImage[];
	files: FirkinFile[];
	year: number | null;
	addon: string;
	creator: string;
	created_at: string;
	updated_at: string;
	version?: number;
	version_hashes?: string[];
	trailers?: FirkinTrailer[];
	reviews?: FirkinReview[];
}
