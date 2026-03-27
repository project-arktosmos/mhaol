import { fetchJson, fetchRaw } from 'ui-lib/transport/fetch-helpers';
import type {
	GameRecommendationRow,
	GameRecommendationsStatus,
	GameBulkEnqueueItem,
	TopRecommendedGame,
	TopRecommendedGameDetail,
	GameRecommendationLabelAssignment
} from 'ui-lib/types/game-recommendations.type';

class GameRecommendationsService {
	async bulkEnqueue(items: GameBulkEnqueueItem[]): Promise<{ enqueued: number }> {
		const res = await fetchRaw('/api/game-recommendations/bulk', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ items })
		});
		if (!res.ok) throw new Error(`Bulk enqueue failed: ${res.status}`);
		return res.json();
	}

	async getStatus(): Promise<GameRecommendationsStatus> {
		return fetchJson<GameRecommendationsStatus>('/api/game-recommendations/status');
	}

	async getForSource(gameId: number): Promise<GameRecommendationRow[]> {
		return fetchJson<GameRecommendationRow[]>(`/api/game-recommendations/${gameId}`);
	}

	async getTop(limit = 50): Promise<TopRecommendedGame[]> {
		return fetchJson<TopRecommendedGame[]>(`/api/game-recommendations/top?limit=${limit}`);
	}

	async getTopDetail(limit = 50): Promise<TopRecommendedGameDetail[]> {
		return fetchJson<TopRecommendedGameDetail[]>(
			`/api/game-recommendations/top-detail?limit=${limit}`
		);
	}

	async getLabelAssignments(wallet: string): Promise<GameRecommendationLabelAssignment[]> {
		return fetchJson<GameRecommendationLabelAssignment[]>(
			`/api/game-recommendations/labels?wallet=${encodeURIComponent(wallet)}`
		);
	}

	async setLabel(wallet: string, gameId: number, labelId: string): Promise<void> {
		const res = await fetchRaw('/api/game-recommendations/labels', {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ wallet, recommendedGameId: gameId, labelId })
		});
		if (!res.ok) throw new Error(`Set label failed: ${res.status}`);
	}

	async removeLabel(wallet: string, gameId: number): Promise<void> {
		const res = await fetchRaw('/api/game-recommendations/labels', {
			method: 'DELETE',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ wallet, recommendedGameId: gameId })
		});
		if (!res.ok) throw new Error(`Remove label failed: ${res.status}`);
	}
}

export const gameRecommendationsService = new GameRecommendationsService();
