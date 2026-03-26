export interface RecommendationLabel {
	id: string;
	name: string;
	emoji: string;
	sortOrder: number;
}

export interface RecommendationLabelAssignment {
	id: string;
	wallet: string;
	recommendedTmdbId: number;
	recommendedMediaType: string;
	labelId: string;
	createdAt: string;
}
