import { fetchJson, fetchRaw } from 'ui-lib/transport/fetch-helpers';
import type {
	RecommendationRow,
	RecommendationsStatus,
	BulkEnqueueItem,
	BulkEnqueueResponse,
	TopRecommendedMovie,
	TopRecommendedMovieDetail,
	TopGenre
} from 'ui-lib/types/recommendations.type';

class RecommendationsService {
	async bulkEnqueue(items: BulkEnqueueItem[]): Promise<BulkEnqueueResponse> {
		const res = await fetchRaw('/api/recommendations/bulk', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ items })
		});
		if (!res.ok) throw new Error(`Bulk enqueue failed: ${res.status}`);
		return res.json();
	}

	async getStatus(mediaType?: string): Promise<RecommendationsStatus> {
		const params = mediaType ? `?mediaType=${mediaType}` : '';
		return fetchJson<RecommendationsStatus>(`/api/recommendations/status${params}`);
	}

	async getForSource(tmdbId: number, mediaType: string): Promise<RecommendationRow[]> {
		return fetchJson<RecommendationRow[]>(`/api/recommendations/${mediaType}/${tmdbId}`);
	}

	async getTopMovies(mediaType?: string, limit = 50): Promise<TopRecommendedMovie[]> {
		const params = new URLSearchParams({ limit: String(limit) });
		if (mediaType) params.set('mediaType', mediaType);
		return fetchJson<TopRecommendedMovie[]>(`/api/recommendations/top-movies?${params}`);
	}

	async getTopMoviesDetail(mediaType?: string, limit = 50): Promise<TopRecommendedMovieDetail[]> {
		const params = new URLSearchParams({ limit: String(limit) });
		if (mediaType) params.set('mediaType', mediaType);
		return fetchJson<TopRecommendedMovieDetail[]>(
			`/api/recommendations/top-movies-detail?${params}`
		);
	}

	async getTopGenres(limit = 50): Promise<TopGenre[]> {
		return fetchJson<TopGenre[]>(`/api/recommendations/top-genres?limit=${limit}`);
	}
}

export const recommendationsService = new RecommendationsService();
