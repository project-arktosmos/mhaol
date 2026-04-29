import { searchRecordings, searchArtists, searchReleaseGroups } from 'addons/musicbrainz';
import { searchBooks } from 'addons/openlibrary';
import { TorrentCategory } from 'addons/torrent-search-thepiratebay/types';
import type {
	Artist,
	DocumentSource,
	DocumentType,
	FileEntry,
	ImageMeta
} from './documents.service';

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

export function matchTorrentsForResult(
	result: SearchResultItem,
	torrents: TorrentResultItem[]
): TorrentResultItem[] {
	const targetWords = normalizeForMatch(result.title)
		.split(' ')
		.filter((w) => w.length > 1);
	if (targetWords.length === 0) return [];
	const matches: TorrentResultItem[] = [];
	for (const t of torrents) {
		const torrentTitle = normalizeForMatch(t.parsedTitle || t.title);
		if (!torrentTitle) continue;
		const hits = targetWords.filter((w) => torrentTitle.includes(w)).length;
		if (hits / targetWords.length < 0.7) continue;
		if (result.year != null && t.year != null && t.year !== result.year) continue;
		matches.push(t);
	}
	matches.sort((a, b) => b.seeders - a.seeders);
	return matches;
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

export async function searchSource(
	source: DocumentSource,
	type: DocumentType,
	query: string
): Promise<SearchResultItem[]> {
	const trimmed = query.trim();
	if (!trimmed) return [];

	switch (source) {
		case 'tmdb':
			return searchTmdb(type, trimmed);
		case 'musicbrainz':
			return searchMusicBrainz(type, trimmed);
		case 'openlibrary':
			return searchOpenLibrary(trimmed);
		default:
			throw new Error(`search not yet supported for source "${source}"`);
	}
}

function tpbCategoryFor(type: DocumentType): TorrentCategory {
	switch (type) {
		case 'album':
		case 'track':
			return TorrentCategory.Audio;
		case 'movie':
		case 'tv show':
		case 'tv season':
		case 'tv episode':
		case 'youtube video':
		case 'youtube channel':
		case 'image':
			return TorrentCategory.Video;
		case 'game':
			return TorrentCategory.Games;
		case 'book':
			return TorrentCategory.Other;
	}
}

export async function searchTorrents(
	type: DocumentType,
	query: string
): Promise<TorrentResultItem[]> {
	const trimmed = query.trim();
	if (!trimmed) return [];
	const res = await fetch('/api/search/torrents', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify({ query: trimmed, category: tpbCategoryFor(type) })
	});
	if (!res.ok) throw new Error(await parseError(res));
	const raw = (await res.json()) as Omit<TorrentResultItem, 'parsedTitle' | 'year' | 'quality'>[];
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

async function searchTmdb(type: DocumentType, query: string): Promise<SearchResultItem[]> {
	const res = await fetch('/api/search/tmdb', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify({ type, query })
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

async function searchMusicBrainz(type: DocumentType, query: string): Promise<SearchResultItem[]> {
	if (type === 'track') {
		const res = await searchRecordings(query);
		return res.recordings.map((rec) => {
			const recAny = rec as unknown as { 'first-release-date'?: string };
			return {
				title: rec.title,
				description: rec.disambiguation ?? '',
				artists: mbArtistCreditsToArtists(rec['artist-credit'] ?? []),
				images: [],
				files: [],
				year: parseYear(recAny['first-release-date']),
				externalId: rec.id,
				raw: rec
			};
		});
	}
	if (type === 'album') {
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
	const res = await searchArtists(query);
	return res.artists.map((a) => {
		const aAny = a as unknown as { 'life-span'?: { begin?: string } };
		return {
			title: a.name,
			description: [a.disambiguation, a.country, a.type]
				.filter((s): s is string => Boolean(s))
				.join(' · '),
			artists: [
				{
					name: a.name,
					url: `https://musicbrainz.org/artist/${a.id}`
				}
			],
			images: [],
			files: [],
			year: parseYear(aAny['life-span']?.begin),
			externalId: a.id,
			raw: a
		};
	});
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
		const artist: Artist = { name };
		if (c.artist?.id) artist.url = `https://musicbrainz.org/artist/${c.artist.id}`;
		out.push(artist);
	}
	return out;
}

async function searchOpenLibrary(query: string): Promise<SearchResultItem[]> {
	const res = await searchBooks(query);
	return (res?.docs ?? []).map((doc) => {
		const authorNames = doc.author_name ?? [];
		const authorKeys = doc.author_key ?? [];
		const artists: Artist[] = authorNames.map((name, i) => ({
			name,
			url: authorKeys[i] ? `https://openlibrary.org/authors/${authorKeys[i]}` : undefined,
			imageUrl: authorKeys[i]
				? `https://covers.openlibrary.org/a/olid/${authorKeys[i]}-L.jpg`
				: undefined
		}));
		const images: ImageMeta[] = doc.cover_i
			? [
					{
						url: `https://covers.openlibrary.org/b/id/${doc.cover_i}-L.jpg`,
						mimeType: 'image/jpeg',
						fileSize: 0,
						width: 0,
						height: 0
					}
				]
			: [];
		const description = [
			doc.first_publish_year ? String(doc.first_publish_year) : null,
			doc.publisher?.[0],
			doc.number_of_pages_median ? `${doc.number_of_pages_median}p` : null
		]
			.filter((s): s is string => Boolean(s))
			.join(' · ');
		return {
			title: doc.title,
			description,
			artists,
			images,
			files: [],
			year: doc.first_publish_year ?? null,
			externalId: doc.key,
			raw: doc
		};
	});
}
