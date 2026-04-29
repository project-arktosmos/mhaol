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
	externalId?: string;
	raw: unknown;
}

export interface TorrentResultItem {
	title: string;
	description: string;
	magnetLink: string;
	infoHash: string;
	raw: unknown;
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
	return (await res.json()) as TorrentResultItem[];
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
		return res.recordings.map((rec) => ({
			title: rec.title,
			description: rec.disambiguation ?? '',
			artists: mbArtistCreditsToArtists(rec['artist-credit'] ?? []),
			images: [],
			files: [],
			externalId: rec.id,
			raw: rec
		}));
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
			externalId: rg.id,
			raw: rg
		}));
	}
	const res = await searchArtists(query);
	return res.artists.map((a) => ({
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
		externalId: a.id,
		raw: a
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
			externalId: doc.key,
			raw: doc
		};
	});
}
