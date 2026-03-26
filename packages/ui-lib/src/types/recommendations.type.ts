export interface RecommendationRow {
	id: number;
	sourceTmdbId: number;
	sourceMediaType: string;
	recommendedTmdbId: number;
	recommendedMediaType: string;
	title: string | null;
	data: Record<string, unknown>;
	fetchedAt: string;
}

export interface RecommendationsStatus {
	pending: number;
	running: number;
	completed: number;
	failed: number;
	total: number;
}

export interface BulkEnqueueItem {
	tmdbId: number;
	mediaType: 'movie' | 'tv';
}

export interface BulkEnqueueResponse {
	enqueued: number;
}
