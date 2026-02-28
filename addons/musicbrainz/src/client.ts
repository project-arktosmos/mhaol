import type {
	MusicBrainzSearchResponse,
	MusicBrainzArtist,
	MusicBrainzReleaseGroup,
	MusicBrainzRelease,
	MusicBrainzRecording
} from './types.js';
import { musicbrainzRateLimiter } from './rate-limiter.js';

const MUSICBRAINZ_BASE_URL = 'https://musicbrainz.org/ws/2';
const USER_AGENT = 'MhaolMedia/1.0.0 (https://github.com/mhaol)';

async function musicbrainzFetch<T>(
	endpoint: string,
	params: Record<string, string> = {}
): Promise<T | null> {
	return musicbrainzRateLimiter.enqueue(async () => {
		const searchParams = new URLSearchParams({
			...params,
			fmt: 'json'
		});

		const url = `${MUSICBRAINZ_BASE_URL}${endpoint}?${searchParams.toString()}`;

		const response = await fetch(url, {
			headers: {
				Accept: 'application/json',
				'User-Agent': USER_AGENT
			}
		});

		if (!response.ok) {
			if (response.status === 404) return null;
			if (response.status === 503) throw new Error('503 Service Unavailable');
			return null;
		}

		return await response.json();
	});
}

// Search functions

export async function searchArtists(
	query: string,
	limit: number = 25,
	offset: number = 0
): Promise<MusicBrainzSearchResponse<MusicBrainzArtist> | null> {
	return musicbrainzFetch<MusicBrainzSearchResponse<MusicBrainzArtist>>('/artist', {
		query,
		limit: limit.toString(),
		offset: offset.toString()
	});
}

export async function searchReleaseGroups(
	query: string,
	limit: number = 25,
	offset: number = 0
): Promise<MusicBrainzSearchResponse<MusicBrainzReleaseGroup> | null> {
	return musicbrainzFetch<MusicBrainzSearchResponse<MusicBrainzReleaseGroup>>('/release-group', {
		query,
		limit: limit.toString(),
		offset: offset.toString()
	});
}

export async function searchRecordings(
	query: string,
	limit: number = 25,
	offset: number = 0
): Promise<MusicBrainzSearchResponse<MusicBrainzRecording> | null> {
	return musicbrainzFetch<MusicBrainzSearchResponse<MusicBrainzRecording>>('/recording', {
		query,
		limit: limit.toString(),
		offset: offset.toString()
	});
}

// Lookup functions

export async function fetchRecording(id: string): Promise<MusicBrainzRecording | null> {
	return musicbrainzFetch<MusicBrainzRecording>(`/recording/${id}`, {
		inc: 'artist-credits+releases+release-groups'
	});
}

export async function fetchArtist(id: string): Promise<MusicBrainzArtist | null> {
	return musicbrainzFetch<MusicBrainzArtist>(`/artist/${id}`, {
		inc: 'tags+release-groups'
	});
}

export async function fetchReleaseGroup(id: string): Promise<MusicBrainzReleaseGroup | null> {
	return musicbrainzFetch<MusicBrainzReleaseGroup>(`/release-group/${id}`, {
		inc: 'artist-credits'
	});
}

export async function fetchReleasesForReleaseGroup(
	releaseGroupId: string
): Promise<MusicBrainzSearchResponse<MusicBrainzRelease> | null> {
	return musicbrainzFetch<MusicBrainzSearchResponse<MusicBrainzRelease>>('/release', {
		'release-group': releaseGroupId,
		inc: 'recordings+media+artist-credits+release-groups',
		status: 'official',
		limit: '1'
	});
}

// TheAudioDB (Artist Images)

export async function fetchArtistImage(musicBrainzId: string): Promise<string | null> {
	try {
		const url = `https://www.theaudiodb.com/api/v1/json/2/artist-mb.php?i=${musicBrainzId}`;
		const response = await fetch(url, {
			headers: { Accept: 'application/json' }
		});

		if (!response.ok) return null;

		const data = await response.json();
		const artist = data?.artists?.[0];
		if (!artist) return null;

		return artist.strArtistThumb || artist.strArtistFanart || null;
	} catch {
		return null;
	}
}
