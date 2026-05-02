import { fetchJson, fetchRaw } from '$transport/fetch-helpers';
import type {
	RecommendationLabel,
	RecommendationLabelAssignment
} from '$types/recommendation-label.type';

class RecommendationLabelsService {
	async getDefinitions(): Promise<RecommendationLabel[]> {
		return fetchJson<RecommendationLabel[]>('/api/recommendation-labels/definitions');
	}

	async getAssignments(wallet: string): Promise<RecommendationLabelAssignment[]> {
		return fetchJson<RecommendationLabelAssignment[]>(
			`/api/recommendation-labels?wallet=${encodeURIComponent(wallet)}`
		);
	}

	async setLabel(
		wallet: string,
		recommendedTmdbId: number,
		recommendedMediaType: string,
		labelId: string
	): Promise<void> {
		await fetchRaw('/api/recommendation-labels', {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ wallet, recommendedTmdbId, recommendedMediaType, labelId })
		});
	}

	async removeLabel(
		wallet: string,
		recommendedTmdbId: number,
		recommendedMediaType: string
	): Promise<void> {
		await fetchRaw('/api/recommendation-labels', {
			method: 'DELETE',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ wallet, recommendedTmdbId, recommendedMediaType })
		});
	}
}

export const recommendationLabelsService = new RecommendationLabelsService();
