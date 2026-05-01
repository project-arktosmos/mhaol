export interface MusicRecommendationRow {
	id: number;
	sourceMbid: string;
	sourceType: string;
	recommendedMbid: string;
	recommendedType: string;
	name: string | null;
	tags: string | null;
	score: number | null;
	level: number;
	data: Record<string, unknown>;
	fetchedAt: string;
}

export interface TopRecommendedArtist {
	mbid: string;
	type: string;
	name: string | null;
	count: number;
	levelCounts: Record<string, number>;
	levelPercentages: Record<string, number>;
	score: number;
	levels: number[];
}

export interface TopRecommendedArtistDetail {
	mbid: string;
	type: string;
	name: string | null;
	count: number;
	minLevel: number;
	data: Record<string, unknown>;
	sources: { mbid: string; type: string; name: string | null }[];
}

export interface MusicBulkEnqueueItem {
	mbid: string;
}

export interface MusicRecommendationsStatus {
	pending: number;
	running: number;
	completed: number;
	failed: number;
	total: number;
}

export interface MusicRecommendationLabelAssignment {
	id: string;
	wallet: string;
	recommendedMbid: string;
	recommendedType: string;
	labelId: string;
	createdAt: string;
}
