export interface RecommendationRow {
	id: number;
	source_tmdb_id: number;
	source_media_type: string;
	recommended_tmdb_id: number;
	recommended_media_type: string;
	title: string | null;
	data: string;
	fetched_at: string;
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
