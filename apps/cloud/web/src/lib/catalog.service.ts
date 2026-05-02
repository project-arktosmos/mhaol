export interface CatalogReview {
	label: string;
	score: number;
	maxScore: number;
	voteCount?: number;
}

export interface CatalogItem {
	id: string;
	title: string;
	year: number | null;
	description: string | null;
	posterUrl: string | null;
	backdropUrl: string | null;
	reviews?: CatalogReview[];
}

export interface CatalogPage {
	items: CatalogItem[];
	page: number;
	totalPages: number;
}

export interface CatalogGenre {
	id: string;
	name: string;
}

/// One browsable addon. Each addon owns a single firkin kind (the kind is
/// implicit in the addon id) — there is no separate `type` parameter
/// anywhere in the catalog API.
export interface CatalogSource {
	id: string;
	label: string;
	kind: string;
	filterLabel: string;
	hasFilter: boolean;
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

export async function listSources(): Promise<CatalogSource[]> {
	const res = await fetch('/api/catalog/sources', { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as CatalogSource[];
}

export async function loadGenres(addon: string): Promise<CatalogGenre[]> {
	const url = `/api/catalog/${encodeURIComponent(addon)}/genres`;
	const res = await fetch(url, { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as CatalogGenre[];
}

export async function loadPopular(
	addon: string,
	options: { filter?: string; page?: number } = {}
): Promise<CatalogPage> {
	const params = new URLSearchParams();
	if (options.filter) params.set('filter', options.filter);
	if (options.page) params.set('page', String(options.page));
	const qs = params.toString();
	const url = `/api/catalog/${encodeURIComponent(addon)}/popular${qs ? `?${qs}` : ''}`;
	const res = await fetch(url, { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as CatalogPage;
}

export async function loadSearch(
	addon: string,
	query: string,
	options: { filter?: string; page?: number; field?: string } = {}
): Promise<CatalogPage> {
	const params = new URLSearchParams();
	params.set('query', query);
	if (options.filter) params.set('filter', options.filter);
	if (options.page) params.set('page', String(options.page));
	if (options.field) params.set('field', options.field);
	const url = `/api/catalog/${encodeURIComponent(addon)}/search?${params.toString()}`;
	const res = await fetch(url, { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as CatalogPage;
}

export async function loadRelated(addon: string, id: string): Promise<CatalogItem[]> {
	const url = `/api/catalog/${encodeURIComponent(addon)}/${encodeURIComponent(id)}/related`;
	const res = await fetch(url, { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as CatalogItem[];
}
