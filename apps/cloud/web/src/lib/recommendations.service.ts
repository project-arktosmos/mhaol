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
	watched: boolean;
	score: number;
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

export async function listRecommendations(address: string): Promise<Recommendation[]> {
	const params = new URLSearchParams({ address });
	const res = await fetch(`/api/recommendations?${params.toString()}`, { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as Recommendation[];
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

export async function updateRecommendation(
	firkinId: string,
	patch: { address: string; watched?: boolean; score?: number }
): Promise<Recommendation> {
	const res = await fetch(`/api/recommendations/${encodeURIComponent(firkinId)}`, {
		method: 'PUT',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify(patch)
	});
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as Recommendation;
}
