import type { CatalogReview } from '$lib/catalog.service';

export interface Recommendation {
	id: string;
	address: string;
	firkinId: string;
	addon: string;
	upstreamId: string;
	title: string;
	year: number | null;
	description: string | null;
	posterUrl: string | null;
	backdropUrl: string | null;
	count: number;
	reviews?: CatalogReview[];
	created_at: string;
	updated_at: string;
}

export interface RecommendationIngestItem {
	addon: string;
	id: string;
	title: string;
	year?: number | null;
	description?: string | null;
	posterUrl?: string | null;
	backdropUrl?: string | null;
	reviews?: CatalogReview[];
}

export interface IngestRequest {
	address: string;
	sourceFirkinId: string;
	items: RecommendationIngestItem[];
}

export interface IngestResponse {
	processed: boolean;
	ingested: number;
}

async function parseError(res: Response): Promise<string> {
	try {
		const data = await res.json();
		if (data && typeof data.error === 'string') return data.error;
	} catch {
		// fall through
	}
	return `HTTP ${res.status}`;
}

export async function listRecommendations(
	address: string,
	options: { excludeActioned?: boolean } = {}
): Promise<Recommendation[]> {
	const params = new URLSearchParams({ address });
	if (options.excludeActioned) params.set('excludeActioned', 'true');
	const res = await fetch(`/api/recommendations?${params.toString()}`, { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as Recommendation[];
}

export type RecommendationAction = 'like' | 'discard' | 'bookmark';

export interface ActionResponse {
	action: RecommendationAction;
	created_at: string;
	updated_at: string;
}

export async function recordRecommendationAction(input: {
	address: string;
	firkinId: string;
	action: RecommendationAction;
}): Promise<ActionResponse> {
	const res = await fetch('/api/recommendations/action', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(input)
	});
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as ActionResponse;
}

export async function ingestRecommendations(req: IngestRequest): Promise<IngestResponse> {
	const res = await fetch('/api/recommendations/ingest', {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(req)
	});
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as IngestResponse;
}
