export interface RecommendationRow {
	id: number;
	sourceTmdbId: number;
	sourceMediaType: string;
	recommendedTmdbId: number;
	recommendedMediaType: string;
	title: string | null;
	genres: string | null;
	level: number;
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

export interface TopRecommendedMovie {
	tmdbId: number;
	mediaType: string;
	title: string | null;
	count: number;
	levelCounts: Record<string, number>;
}

export interface RecommendationSource {
	tmdbId: number;
	mediaType: string;
	title: string | null;
}

export interface TopRecommendedMovieDetail {
	tmdbId: number;
	mediaType: string;
	title: string | null;
	count: number;
	minLevel: number;
	data: Record<string, unknown>;
	sources: RecommendationSource[];
}

export interface TopGenre {
	genre: string;
	count: number;
}
