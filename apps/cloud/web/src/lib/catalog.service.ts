export interface CatalogItem {
	id: string;
	title: string;
	year: number | null;
	description: string | null;
	posterUrl: string | null;
	backdropUrl: string | null;
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

export interface CatalogTypeInfo {
	id: string;
	label: string;
}

export interface CatalogSource {
	id: string;
	label: string;
	types: CatalogTypeInfo[];
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

export async function loadGenres(addon: string, type?: string): Promise<CatalogGenre[]> {
	const params = new URLSearchParams();
	if (type) params.set('type', type);
	const qs = params.toString();
	const url = `/api/catalog/${encodeURIComponent(addon)}/genres${qs ? `?${qs}` : ''}`;
	const res = await fetch(url, { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as CatalogGenre[];
}

export async function loadPopular(
	addon: string,
	options: { type?: string; filter?: string; page?: number } = {}
): Promise<CatalogPage> {
	const params = new URLSearchParams();
	if (options.type) params.set('type', options.type);
	if (options.filter) params.set('filter', options.filter);
	if (options.page) params.set('page', String(options.page));
	const qs = params.toString();
	const url = `/api/catalog/${encodeURIComponent(addon)}/popular${qs ? `?${qs}` : ''}`;
	const res = await fetch(url, { cache: 'no-store' });
	if (!res.ok) throw new Error(await parseError(res));
	return (await res.json()) as CatalogPage;
}
