import { fetchJson, fetchRaw } from 'ui-lib/transport/fetch-helpers';
import type {
	BookRecommendationRow,
	BookRecommendationsStatus,
	BookBulkEnqueueItem,
	TopRecommendedBook,
	TopRecommendedBookDetail,
	BookRecommendationLabelAssignment
} from 'ui-lib/types/book-recommendations.type';

class BookRecommendationsService {
	async bulkEnqueue(items: BookBulkEnqueueItem[]): Promise<{ enqueued: number }> {
		const res = await fetchRaw('/api/book-recommendations/bulk', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ items })
		});
		if (!res.ok) throw new Error(`Bulk enqueue failed: ${res.status}`);
		return res.json();
	}

	async getStatus(): Promise<BookRecommendationsStatus> {
		return fetchJson<BookRecommendationsStatus>('/api/book-recommendations/status');
	}

	async getForSource(key: string): Promise<BookRecommendationRow[]> {
		return fetchJson<BookRecommendationRow[]>(
			`/api/book-recommendations/${encodeURIComponent(key)}`
		);
	}

	async getTop(limit = 50): Promise<TopRecommendedBook[]> {
		return fetchJson<TopRecommendedBook[]>(`/api/book-recommendations/top?limit=${limit}`);
	}

	async getTopDetail(limit = 50): Promise<TopRecommendedBookDetail[]> {
		return fetchJson<TopRecommendedBookDetail[]>(
			`/api/book-recommendations/top-detail?limit=${limit}`
		);
	}

	async getLabelAssignments(wallet: string): Promise<BookRecommendationLabelAssignment[]> {
		return fetchJson<BookRecommendationLabelAssignment[]>(
			`/api/book-recommendations/labels?wallet=${encodeURIComponent(wallet)}`
		);
	}

	async setLabel(wallet: string, key: string, labelId: string): Promise<void> {
		const res = await fetchRaw('/api/book-recommendations/labels', {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				wallet,
				recommendedKey: key,
				labelId
			})
		});
		if (!res.ok) throw new Error(`Set label failed: ${res.status}`);
	}

	async removeLabel(wallet: string, key: string): Promise<void> {
		const res = await fetchRaw('/api/book-recommendations/labels', {
			method: 'DELETE',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				wallet,
				recommendedKey: key
			})
		});
		if (!res.ok) throw new Error(`Remove label failed: ${res.status}`);
	}
}

export const bookRecommendationsService = new BookRecommendationsService();
