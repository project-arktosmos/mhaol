export interface BookRecommendationRow {
	id: number;
	sourceKey: string;
	recommendedKey: string;
	title: string | null;
	authors: string | null;
	subjects: string | null;
	score: number;
	level: number;
	data: Record<string, unknown>;
	fetchedAt: string;
}

export interface TopRecommendedBook {
	key: string;
	title: string | null;
	count: number;
	levelCounts: Record<string, number>;
	levelPercentages: Record<string, number>;
	score: number;
	levels: number[];
}

export interface TopRecommendedBookDetail {
	key: string;
	title: string | null;
	count: number;
	minLevel: number;
	data: Record<string, unknown>;
	sources: { key: string; title: string | null }[];
}

export interface BookBulkEnqueueItem {
	key: string;
}

export interface BookRecommendationsStatus {
	pending: number;
	running: number;
	completed: number;
	failed: number;
	total: number;
}

export interface BookRecommendationLabelAssignment {
	id: string;
	wallet: string;
	recommendedKey: string;
	labelId: string;
	createdAt: string;
}
