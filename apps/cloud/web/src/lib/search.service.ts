import { searchMovies, searchTvShows } from 'addons/tmdb';
import { extractYear } from 'addons/tmdb/transform';
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

const TMDB_API_KEY = (import.meta.env.VITE_TMDB_API_KEY as string | undefined) ?? '';

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

async function searchTmdb(type: DocumentType, query: string): Promise<SearchResultItem[]> {
	if (!TMDB_API_KEY) {
		throw new Error('TMDB API key missing — set VITE_TMDB_API_KEY in your env');
	}
	if (type === 'tv show' || type === 'tv season' || type === 'tv episode') {
		const res = await searchTvShows(TMDB_API_KEY, query);
		return (res?.results ?? []).map((show) => ({
			title: show.name,
			author: extractYear(show.first_air_date) ?? '',
			description: show.overview ?? '',
			externalId: String(show.id)
		}));
	}
	const res = await searchMovies(TMDB_API_KEY, query);
	return (res?.results ?? []).map((movie) => ({
		title: movie.title,
		author: extractYear(movie.release_date) ?? '',
		description: movie.overview ?? '',
		externalId: String(movie.id)
	}));
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
