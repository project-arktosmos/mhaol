export interface SmartPairItem {
	title: string;
	id: string;
	source: string;
}

export interface SmartPairResult {
	sourceId: string;
	sourceTitle: string;
	source: string;
	matched: boolean;
	tmdbId: number | null;
	tmdbTitle: string | null;
	tmdbType: 'movie' | 'tv' | null;
	tmdbYear: string | null;
	tmdbPosterPath: string | null;
	confidence: 'high' | 'medium' | 'low' | 'none';
	accepted: boolean;
}

export interface SmartPairState {
	pairing: boolean;
	results: SmartPairResult[];
	saving: boolean;
	saved: boolean;
	error: string | null;
}
