import type {
	MusicBrainzSearchResponse,
	MusicBrainzArtist,
	MusicBrainzReleaseGroup,
	MusicBrainzRelease,
	MusicBrainzRecording
} from '$types/musicbrainz.type';
import { musicBrainzRateLimiter } from '$utils/rate-limiter.util';

class MusicBrainzService {
	private baseUrl = 'https://musicbrainz.org/ws/2';
	private userAgent = 'MhaolMedia/1.0.0 (https://github.com/mhaol)';

	private async fetch<T>(endpoint: string, params: Record<string, string> = {}): Promise<T | null> {
		try {
			return await musicBrainzRateLimiter.enqueue(async () => {
				const searchParams = new URLSearchParams({
					...params,
					fmt: 'json'
				});

				const url = `${this.baseUrl}${endpoint}?${searchParams.toString()}`;

				const response = await fetch(url, {
					headers: {
						Accept: 'application/json',
						'User-Agent': this.userAgent
					}
				});

				if (!response.ok) {
					if (response.status === 404) {
						return null;
					}
					if (response.status === 503) {
						throw new Error('503 Service Unavailable');
					}
					return null;
				}

				return await response.json();
			});
		} catch (error) {
			throw error;
		}
	}

	// =========================================================================
	// Search Methods
	// =========================================================================

	async searchArtists(
		query: string,
		limit: number = 25,
		offset: number = 0
	): Promise<MusicBrainzSearchResponse<MusicBrainzArtist> | null> {
		return this.fetch<MusicBrainzSearchResponse<MusicBrainzArtist>>('/artist', {
			query,
			limit: limit.toString(),
			offset: offset.toString()
		});
	}

	async searchReleaseGroups(
		query: string,
		limit: number = 25,
		offset: number = 0
	): Promise<MusicBrainzSearchResponse<MusicBrainzReleaseGroup> | null> {
		return this.fetch<MusicBrainzSearchResponse<MusicBrainzReleaseGroup>>('/release-group', {
			query,
			limit: limit.toString(),
			offset: offset.toString()
		});
	}

	async searchRecordings(
		query: string,
		limit: number = 25,
		offset: number = 0
	): Promise<MusicBrainzSearchResponse<MusicBrainzRecording> | null> {
		return this.fetch<MusicBrainzSearchResponse<MusicBrainzRecording>>('/recording', {
			query,
			limit: limit.toString(),
			offset: offset.toString()
		});
	}

	// =========================================================================
	// Lookup Methods
	// =========================================================================

	async fetchArtist(id: string): Promise<MusicBrainzArtist | null> {
		return this.fetch<MusicBrainzArtist>(`/artist/${id}`, {
			inc: 'tags+release-groups'
		});
	}

	async fetchReleaseGroup(id: string): Promise<MusicBrainzReleaseGroup | null> {
		return this.fetch<MusicBrainzReleaseGroup>(`/release-group/${id}`, {
			inc: 'artist-credits'
		});
	}

	async fetchReleasesForReleaseGroup(
		releaseGroupId: string
	): Promise<MusicBrainzSearchResponse<MusicBrainzRelease> | null> {
		return this.fetch<MusicBrainzSearchResponse<MusicBrainzRelease>>('/release', {
			'release-group': releaseGroupId,
			inc: 'recordings+media+artist-credits+release-groups',
			status: 'official',
			limit: '1'
		});
	}

	// =========================================================================
	// TheAudioDB (Artist Images)
	// =========================================================================

	async fetchArtistImage(musicBrainzId: string): Promise<string | null> {
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

	get pendingRequests(): number {
		return musicBrainzRateLimiter.queueLength;
	}
}

export const musicBrainzService = new MusicBrainzService();
