export interface WebSurfxResult {
	url: string;
	title: string;
	snippet: string | null;
	domain: string | null;
	publishedDate: string | null;
	provider: string | null;
}

export interface WebSurfxSearchResponse {
	results: WebSurfxResult[];
	provider: string;
}

export interface WebSurfxState {
	query: string;
	searching: boolean;
	results: WebSurfxResult[];
	provider: string | null;
	error: string | null;
}
