export interface GameRecommendationRow {
	id: number;
	sourceGameId: number;
	recommendedGameId: number;
	title: string | null;
	genre: string | null;
	consoleId: number | null;
	consoleName: string | null;
	score: number;
	level: number;
	data: Record<string, unknown>;
	fetchedAt: string;
}

export interface TopRecommendedGame {
	gameId: number;
	title: string | null;
	count: number;
	levelCounts: Record<string, number>;
	levelPercentages: Record<string, number>;
	score: number;
	levels: number[];
}

export interface TopRecommendedGameDetail {
	gameId: number;
	title: string | null;
	count: number;
	minLevel: number;
	data: Record<string, unknown>;
	sources: { gameId: number; title: string | null }[];
}

export interface GameBulkEnqueueItem {
	gameId: number;
}

export interface GameRecommendationsStatus {
	pending: number;
	running: number;
	completed: number;
	failed: number;
	total: number;
}

export interface GameRecommendationLabelAssignment {
	id: string;
	wallet: string;
	recommendedGameId: number;
	labelId: string;
	createdAt: string;
}
