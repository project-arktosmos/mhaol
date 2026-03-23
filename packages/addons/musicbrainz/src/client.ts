import type { MusicBrainzRecording, MusicBrainzArtist, MusicBrainzReleaseGroup } from './types.js';

const BASE_URL = 'https://musicbrainz.org/ws/2';
const HEADERS = {
	Accept: 'application/json',
	'User-Agent': 'Mhaol/0.0.1 (https://github.com/project-arktosmos/mhaol)'
};

async function fetchMusicBrainz<T>(endpoint: string, query: string): Promise<T> {
	const url = `${BASE_URL}/${endpoint}?query=${encodeURIComponent(query)}&fmt=json&limit=20`;
	const res = await fetch(url, { headers: HEADERS });
	if (!res.ok) throw new Error(`MusicBrainz API error: ${res.status}`);
	return res.json();
}

export async function searchRecordings(
	query: string
): Promise<{ recordings: MusicBrainzRecording[] }> {
	return fetchMusicBrainz('recording', query);
}

export async function searchArtists(query: string): Promise<{ artists: MusicBrainzArtist[] }> {
	return fetchMusicBrainz('artist', query);
}

export async function searchReleaseGroups(
	query: string
): Promise<{ 'release-groups': MusicBrainzReleaseGroup[] }> {
	return fetchMusicBrainz('release-group', query);
}
