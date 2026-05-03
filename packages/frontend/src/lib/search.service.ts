import { searchReleaseGroups } from 'addons/musicbrainz';
import { TorrentCategory } from 'addons/torrent-search-thepiratebay/types';
import type { Artist, FirkinAddon, FileEntry, ImageMeta } from './firkins.service';
import { addonKind } from './firkins.service';

export interface SearchResultItem {
	title: string;
	description: string;
	artists: Artist[];
	images: ImageMeta[];
	files: FileEntry[];
	year: number | null;
	externalId?: string;
	raw: unknown;
}

function parseYear(s: string | null | undefined): number | null {
	if (!s) return null;
	const n = parseInt(s.slice(0, 4), 10);
	return Number.isFinite(n) && n >= 1000 && n <= 9999 ? n : null;
}

export interface TorrentResultItem {
	title: string;
	parsedTitle: string;
	year: number | null;
	quality: string | null;
	seeders: number;
	leechers: number;
	sizeBytes: number;
	description: string;
	magnetLink: string;
	infoHash: string;
	raw: unknown;
}

function normalizeForMatch(s: string): string {
	return s
		.toLowerCase()
		.replace(/[^a-z0-9\s]/g, ' ')
		.replace(/\s+/g, ' ')
		.trim();
}

export interface MatchTorrentsOptions {
	/** Skip the year-equality filter — TV torrents are tagged with the season's
	 * air year, not the show's first-aired year, so dropping them by year
	 * mismatch loses every season after S1. */
	skipYearFilter?: boolean;
	/** Drop torrents whose title looks like a TV release (`SxxExx`, `Season N`,
	 * `Complete Series`). Set when the source firkin is a movie so a query
	 * for "Superman" doesn't pull "My Adventures with Superman S02E09" back. */
	excludeTvSeries?: boolean;
}

const TV_RELEASE_RE = /\b(?:s\d{1,2}e\d{1,3}|s\d{1,2}(?!\d)|season\s*\d+|complete\s+series)\b/i;
const LEADING_ARTICLES = new Set(['the', 'a', 'an']);

function stripLeadingArticle(words: string[]): string[] {
	return words.length > 1 && LEADING_ARTICLES.has(words[0]) ? words.slice(1) : words;
}

/// True when the torrent's parsed title begins with the target title's words.
/// "Superman" matches "Superman 2025 1080p" but not "My Adventures with
/// Superman S02E09" — the latter's first word is `my`, not `superman`. A
/// leading article (`the`/`a`/`an`) on either side is ignored so "The Matrix"
/// and "Matrix" align.
function torrentTitleStartsWithTarget(torrentTitle: string, target: string[]): boolean {
	if (target.length === 0) return false;
	const torrentWords = stripLeadingArticle(
		torrentTitle.split(' ').filter((w) => w.length > 0)
	);
	const targetWords = stripLeadingArticle(target);
	if (torrentWords.length < targetWords.length) return false;
	for (let i = 0; i < targetWords.length; i++) {
		if (torrentWords[i] !== targetWords[i]) return false;
	}
	return true;
}

export function matchTorrentsForResult(
	result: SearchResultItem,
	torrents: TorrentResultItem[],
	options: MatchTorrentsOptions = {}
): TorrentResultItem[] {
	const targetWords = normalizeForMatch(result.title)
		.split(' ')
		.filter((w) => w.length > 0);
	if (targetWords.length === 0) return [];
	const matches: TorrentResultItem[] = [];
	for (const t of torrents) {
		if (options.excludeTvSeries && TV_RELEASE_RE.test(t.title)) continue;
		const torrentTitle = normalizeForMatch(t.parsedTitle || t.title);
		if (!torrentTitle) continue;
		if (!torrentTitleStartsWithTarget(torrentTitle, targetWords)) continue;
		if (
			!options.skipYearFilter &&
			result.year != null &&
			t.year != null &&
			t.year !== result.year
		) {
			continue;
		}
		matches.push(t);
	}
	matches.sort((a, b) => b.seeders - a.seeders);
	return matches;
}

/// Detect the season range a torrent's name covers. Returns `{ start, end }`
/// for single-season torrents (`start === end`) or multi-season packs
/// (`Show.S01-S03`, `Season 1-3`). Returns `null` for whole-show / complete
/// packs that don't tag a specific season.
///
/// Episode-specific torrents like `S01E03` are still classified as season 1.
export function parseTorrentSeasons(name: string): { start: number; end: number } | null {
	const range = name.match(
		/\b(?:s|season[\s._-]*)(\d{1,2})\s*(?:-|–|to)\s*(?:s|season[\s._-]*)?(\d{1,2})\b/i
	);
	if (range) {
		const a = parseInt(range[1], 10);
		const b = parseInt(range[2], 10);
		if (Number.isFinite(a) && Number.isFinite(b)) {
			return { start: Math.min(a, b), end: Math.max(a, b) };
		}
	}
	const single = name.match(/\b(?:s|season[\s._-]*)(\d{1,2})\b/i);
	if (single) {
		const n = parseInt(single[1], 10);
		if (Number.isFinite(n)) return { start: n, end: n };
	}
	return null;
}

export function formatSizeBytes(bytes: number): string {
	if (!bytes || bytes <= 0) return '';
	const units = ['B', 'KB', 'MB', 'GB', 'TB'];
	let n = bytes;
	let i = 0;
	while (n >= 1024 && i < units.length - 1) {
		n /= 1024;
		i++;
	}
	return `${n.toFixed(n >= 100 || i === 0 ? 0 : 1)} ${units[i]}`;
}

const TORRENT_QUALITY_PATTERNS: [RegExp, string][] = [
	[/\b2160p\b/i, '2160p'],
	[/\bUHD\b/i, '4K UHD'],
	[/\b4K\b/i, '4K'],
	[/\b1080p\b/i, '1080p'],
	[/\b720p\b/i, '720p'],
	[/\b480p\b/i, '480p'],
	[/\b360p\b/i, '360p']
];

export function parseTorrentName(name: string): {
	parsedTitle: string;
	year: number | null;
	quality: string | null;
} {
	const yearMatch = name.match(/[\s.([](\d{4})[\s.)\]]/);
	const year = yearMatch ? parseInt(yearMatch[1], 10) : null;

	let quality: string | null = null;
	let qualityIdx = -1;
	for (const [re, label] of TORRENT_QUALITY_PATTERNS) {
		const m = name.match(re);
		if (m && m.index !== undefined) {
			quality = label;
			qualityIdx = m.index;
			break;
		}
	}

	const yearIdx = yearMatch?.index ?? -1;
	let cutIdx = -1;
	if (yearIdx > 0) cutIdx = yearIdx;
	if (qualityIdx > 0 && (cutIdx < 0 || qualityIdx < cutIdx)) cutIdx = qualityIdx;

	let parsedTitle = cutIdx > 0 ? name.slice(0, cutIdx) : name;
	parsedTitle = parsedTitle.replace(/[._]/g, ' ').replace(/\s+/g, ' ').trim();
	if (!parsedTitle) parsedTitle = name;

	return { parsedTitle, year, quality };
}

export async function searchAddon(addon: string, query: string): Promise<SearchResultItem[]> {
	const trimmed = query.trim();
	if (!trimmed) return [];

	if (addon === 'tmdb-movie' || addon === 'tmdb-tv') {
		return searchTmdb(addon, trimmed);
	}
	if (addon === 'musicbrainz') {
		return searchMusicBrainz(trimmed);
	}
	throw new Error(`search not yet supported for addon "${addon}"`);
}

export function isTorrentSearchableAddon(addon: string): boolean {
	return tpbCategoryFor(addon) !== null;
}

export function tpbCategoryFor(addon: string): TorrentCategory | null {
	const kind = addonKind(addon);
	switch (kind) {
		case 'album':
			return TorrentCategory.Audio;
		case 'movie':
		case 'tv show':
		case 'youtube video':
			return TorrentCategory.Video;
		case 'game':
			return TorrentCategory.Games;
		case 'book':
			return TorrentCategory.Other;
		case null:
			return null;
	}
}

const TORRENT_CACHE_STORAGE_KEY = 'mhaol-cloud:torrent-search-cache';
const TORRENT_CACHE_TTL_MS = 24 * 60 * 60 * 1000;
const TORRENT_CACHE_MAX_ENTRIES = 200;

type RawTorrentResult = Omit<TorrentResultItem, 'parsedTitle' | 'year' | 'quality'>;

interface TorrentCacheEntry {
	ts: number;
	data: RawTorrentResult[];
}

function loadTorrentCache(): Record<string, TorrentCacheEntry> {
	if (typeof localStorage === 'undefined') return {};
	try {
		const raw = localStorage.getItem(TORRENT_CACHE_STORAGE_KEY);
		if (!raw) return {};
		const parsed = JSON.parse(raw);
		return parsed && typeof parsed === 'object' ? parsed : {};
	} catch {
		return {};
	}
}

function saveTorrentCache(cache: Record<string, TorrentCacheEntry>): void {
	if (typeof localStorage === 'undefined') return;
	try {
		localStorage.setItem(TORRENT_CACHE_STORAGE_KEY, JSON.stringify(cache));
	} catch {
		// quota exceeded or unavailable — drop silently
	}
}

function torrentCacheKey(category: string, query: string): string {
	return `${category}::${query.toLowerCase()}`;
}

export async function searchTorrents(addon: string, query: string): Promise<TorrentResultItem[]> {
	const trimmed = query.trim();
	if (!trimmed) return [];
	const category = tpbCategoryFor(addon);
	if (category === null) return [];
	const key = torrentCacheKey(category, trimmed);
	const now = Date.now();
	const cache = loadTorrentCache();
	const hit = cache[key];
	let raw: RawTorrentResult[];
	if (hit && now - hit.ts < TORRENT_CACHE_TTL_MS) {
		raw = hit.data;
	} else {
		const res = await fetch('/api/search/torrents', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ query: trimmed, category })
		});
		if (!res.ok) throw new Error(await parseError(res));
		raw = (await res.json()) as RawTorrentResult[];
		cache[key] = { ts: now, data: raw };
		const keys = Object.keys(cache);
		if (keys.length > TORRENT_CACHE_MAX_ENTRIES) {
			const trimmedKeys = keys
				.map((k) => [k, cache[k].ts] as const)
				.sort((a, b) => a[1] - b[1])
				.slice(0, keys.length - TORRENT_CACHE_MAX_ENTRIES)
				.map(([k]) => k);
			for (const k of trimmedKeys) delete cache[k];
		}
		saveTorrentCache(cache);
	}
	return raw.map((t) => ({ ...t, ...parseTorrentName(t.title) }));
}

async function parseError(res: Response): Promise<string> {
	try {
		const data = await res.json();
		if (data && typeof data.error === 'string') return data.error;
	} catch {
		// fall through
	}
	return `HTTP ${res.status}`;
}

async function searchTmdb(addon: FirkinAddon, query: string): Promise<SearchResultItem[]> {
	const res = await fetch('/api/search/tmdb', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify({ addon, query })
	});
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as SearchResultItem[];
}

export async function fetchTmdbEpisodeTitles(showId: string): Promise<string[]> {
	const res = await fetch('/api/search/tmdb/episodes', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify({ id: showId })
	});
	if (!res.ok) throw new Error(await parseError(res));
	const data = (await res.json()) as { title: string }[];
	return data.map((e) => e.title);
}

export async function fetchAlbumTrackTitles(releaseGroupId: string): Promise<string[]> {
	const rgRes = await fetch(
		`https://musicbrainz.org/ws/2/release-group/${encodeURIComponent(releaseGroupId)}?inc=releases&fmt=json`,
		{ headers: { Accept: 'application/json' } }
	);
	if (!rgRes.ok) throw new Error(`MusicBrainz returned ${rgRes.status}`);
	const rg = (await rgRes.json()) as { releases?: { id: string }[] };
	const releaseId = rg.releases?.[0]?.id;
	if (!releaseId) return [];
	const relRes = await fetch(
		`https://musicbrainz.org/ws/2/release/${encodeURIComponent(releaseId)}?inc=recordings&fmt=json`,
		{ headers: { Accept: 'application/json' } }
	);
	if (!relRes.ok) throw new Error(`MusicBrainz returned ${relRes.status}`);
	const rel = (await relRes.json()) as {
		media?: { tracks?: { title: string; position?: number }[] }[];
	};
	const titles: string[] = [];
	for (const m of rel.media ?? []) {
		for (const t of m.tracks ?? []) {
			titles.push(t.title);
		}
	}
	return titles;
}

async function searchMusicBrainz(query: string): Promise<SearchResultItem[]> {
	const res = await searchReleaseGroups(query);
	return res['release-groups'].map((rg) => ({
		title: rg.title,
		description: [rg['primary-type'], rg['first-release-date']]
			.filter((s): s is string => Boolean(s))
			.join(' · '),
		artists: mbArtistCreditsToArtists(rg['artist-credit'] ?? []),
		images: [
			{
				url: `https://coverartarchive.org/release-group/${rg.id}/front`,
				mimeType: 'image/jpeg',
				fileSize: 0,
				width: 0,
				height: 0
			}
		],
		files: [],
		year: parseYear(rg['first-release-date']),
		externalId: rg.id,
		raw: rg
	}));
}

interface MbArtistCredit {
	name?: string;
	artist?: { id: string; name: string };
	joinphrase?: string;
}

function mbArtistCreditsToArtists(credits: MbArtistCredit[]): Artist[] {
	const out: Artist[] = [];
	for (const c of credits) {
		const name = c.artist?.name ?? c.name ?? '';
		if (!name) continue;
		out.push({ name, role: 'Artist' });
	}
	return out;
}
