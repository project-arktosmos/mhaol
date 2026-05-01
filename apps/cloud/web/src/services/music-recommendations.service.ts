import { fetchJson, fetchRaw } from '$transport/fetch-helpers';
import type {
	MusicRecommendationRow,
	MusicRecommendationsStatus,
	MusicBulkEnqueueItem,
	TopRecommendedArtist,
	TopRecommendedArtistDetail,
	MusicRecommendationLabelAssignment
} from '$types/music-recommendations.type';

class MusicRecommendationsService {
	async bulkEnqueue(items: MusicBulkEnqueueItem[]): Promise<{ enqueued: number }> {
		const res = await fetchRaw('/api/music-recommendations/bulk', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ items })
		});
		if (!res.ok) throw new Error(`Bulk enqueue failed: ${res.status}`);
		return res.json();
	}

	async getStatus(): Promise<MusicRecommendationsStatus> {
		return fetchJson<MusicRecommendationsStatus>('/api/music-recommendations/status');
	}

	async getForSource(mbid: string): Promise<MusicRecommendationRow[]> {
		return fetchJson<MusicRecommendationRow[]>(`/api/music-recommendations/${mbid}`);
	}

	async getTop(limit = 50): Promise<TopRecommendedArtist[]> {
		return fetchJson<TopRecommendedArtist[]>(`/api/music-recommendations/top?limit=${limit}`);
	}

	async getTopDetail(limit = 50): Promise<TopRecommendedArtistDetail[]> {
		return fetchJson<TopRecommendedArtistDetail[]>(
			`/api/music-recommendations/top-detail?limit=${limit}`
		);
	}

	async getLabelAssignments(wallet: string): Promise<MusicRecommendationLabelAssignment[]> {
		return fetchJson<MusicRecommendationLabelAssignment[]>(
			`/api/music-recommendations/labels?wallet=${encodeURIComponent(wallet)}`
		);
	}

	async setLabel(wallet: string, mbid: string, type: string, labelId: string): Promise<void> {
		const res = await fetchRaw('/api/music-recommendations/labels', {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				wallet,
				recommendedMbid: mbid,
				recommendedType: type,
				labelId
			})
		});
		if (!res.ok) throw new Error(`Set label failed: ${res.status}`);
	}

	async removeLabel(wallet: string, mbid: string, type: string): Promise<void> {
		const res = await fetchRaw('/api/music-recommendations/labels', {
			method: 'DELETE',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				wallet,
				recommendedMbid: mbid,
				recommendedType: type
			})
		});
		if (!res.ok) throw new Error(`Remove label failed: ${res.status}`);
	}
}

export const musicRecommendationsService = new MusicRecommendationsService();
