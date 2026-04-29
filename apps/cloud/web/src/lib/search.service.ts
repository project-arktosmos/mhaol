import { searchRecordings, searchArtists, searchReleaseGroups } from 'addons/musicbrainz';
import { formatArtistCredits } from 'addons/musicbrainz/transform';
import { searchBooks } from 'addons/openlibrary';
import { search as searchPirateBay } from 'addons/torrent-search-thepiratebay';
import type { DocumentSource, DocumentType } from './documents.service';

export interface SearchResultItem {
	title: string;
	author: string;
	description: string;
	externalId?: string;
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
		case 'torrent-search-thepiratebay':
			return searchTpb(trimmed);
		default:
			throw new Error(`search not yet supported for source "${source}"`);
	}
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

async function searchMusicBrainz(type: DocumentType, query: string): Promise<SearchResultItem[]> {
	if (type === 'track') {
		const res = await searchRecordings(query);
		return res.recordings.map((rec) => ({
			title: rec.title,
			author: formatArtistCredits(rec['artist-credit'] ?? []),
			description: '',
			externalId: rec.id
		}));
	}
	if (type === 'album') {
		const res = await searchReleaseGroups(query);
		return res['release-groups'].map((rg) => ({
			title: rg.title,
			author: formatArtistCredits(rg['artist-credit'] ?? []),
			description: rg['primary-type'] ?? '',
			externalId: rg.id
		}));
	}
	const res = await searchArtists(query);
	return res.artists.map((a) => ({
		title: a.name,
		author: a.country ?? '',
		description: a.disambiguation ?? '',
		externalId: a.id
	}));
}

async function searchOpenLibrary(query: string): Promise<SearchResultItem[]> {
	const res = await searchBooks(query);
	return (res?.docs ?? []).map((doc) => ({
		title: doc.title,
		author: (doc.author_name ?? []).join(', '),
		description: doc.first_publish_year ? String(doc.first_publish_year) : '',
		externalId: doc.key
	}));
}

async function searchTpb(query: string): Promise<SearchResultItem[]> {
	const results = await searchPirateBay(query);
	return results.map((r) => ({
		title: r.name,
		author: r.uploadedBy ?? '',
		description: `${r.seeders} seeders · ${r.leechers} leechers · ${r.size} bytes`,
		externalId: r.infoHash
	}));
}
