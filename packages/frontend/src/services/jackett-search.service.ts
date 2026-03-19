import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from 'frontend/lib/api-base';

export interface JackettSearchResult {
	id: string;
	name: string;
	infoHash: string;
	magnetLink: string;
	seeders: number;
	leechers: number;
	size: number;
	category: string;
	uploadedAt: number;
	tracker: string;
}

export interface JackettIndexer {
	id: string;
	name: string;
}

export enum JackettCategory {
	All = '',
	Movies = '2000',
	TV = '5000',
	Audio = '3000',
	PC = '4000',
	Console = '1000',
	Books = '8000',
	Other = '7000'
}

export const JACKETT_CATEGORY_LABELS: Record<JackettCategory, string> = {
	[JackettCategory.All]: 'All',
	[JackettCategory.Movies]: 'Movies',
	[JackettCategory.TV]: 'TV',
	[JackettCategory.Audio]: 'Audio',
	[JackettCategory.PC]: 'PC',
	[JackettCategory.Console]: 'Console',
	[JackettCategory.Books]: 'Books',
	[JackettCategory.Other]: 'Other'
};

export interface JackettSearchFilters {
	category: JackettCategory;
	tracker: string;
}

interface JackettSearchState {
	query: string;
	filters: JackettSearchFilters;
	searching: boolean;
	results: JackettSearchResult[];
	sort: { field: JackettSortField; direction: 'asc' | 'desc' };
	error: string | null;
	addingTorrents: Set<string>;
	indexers: JackettIndexer[];
}

export type JackettSortField = 'seeders' | 'leechers' | 'size' | 'name' | 'uploadedAt';

const initialState: JackettSearchState = {
	query: '',
	filters: { category: JackettCategory.All, tracker: '' },
	searching: false,
	results: [],
	sort: { field: 'seeders', direction: 'desc' },
	error: null,
	addingTorrents: new Set(),
	indexers: []
};

class JackettSearchService {
	public state: Writable<JackettSearchState> = writable(initialState);

	async search(query: string, filters: JackettSearchFilters): Promise<void> {
		if (!browser || !query.trim()) return;

		this.state.update((s) => ({
			...s,
			query: query.trim(),
			filters,
			searching: true,
			error: null,
			results: []
		}));

		try {
			const params = new URLSearchParams({ q: query.trim() });
			if (filters.category) {
				params.set('cat', filters.category);
			}
			if (filters.tracker) {
				params.set('tracker', filters.tracker);
			}
			const response = await fetch(apiUrl(`/api/jackett/search?${params}`));

			if (!response.ok) {
				const body = await response.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
			}

			const body: { results: JackettSearchResult[]; indexers: JackettIndexer[] } =
				await response.json();

			this.state.update((s) => ({
				...s,
				searching: false,
				results: body.results,
				indexers: body.indexers.length > 0 ? body.indexers : s.indexers
			}));
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				searching: false,
				error: `Search failed: ${errorMsg}`
			}));
		}
	}

	toggleSort(field: JackettSortField): void {
		this.state.update((s) => {
			if (s.sort.field === field) {
				return {
					...s,
					sort: { field, direction: s.sort.direction === 'desc' ? 'asc' : 'desc' }
				};
			}
			return {
				...s,
				sort: { field, direction: 'desc' }
			};
		});
	}

	markAdding(infoHash: string): void {
		this.state.update((s) => {
			const next = new Set(s.addingTorrents);
			next.add(infoHash);
			return { ...s, addingTorrents: next };
		});
	}

	unmarkAdding(infoHash: string): void {
		this.state.update((s) => {
			const next = new Set(s.addingTorrents);
			next.delete(infoHash);
			return { ...s, addingTorrents: next };
		});
	}

	clearResults(): void {
		this.state.update((s) => ({
			...s,
			results: [],
			error: null
		}));
	}
}

export const jackettSearchService = new JackettSearchService();
