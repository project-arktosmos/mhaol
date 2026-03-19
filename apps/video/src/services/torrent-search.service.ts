import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from '$lib/api-base';
import {
	TorrentCategory,
	type TorrentSearchResult,
	type TorrentSearchSort
} from 'addons/torrent-search-thepiratebay/types';

interface TorrentSearchState {
	query: string;
	category: TorrentCategory;
	searching: boolean;
	results: TorrentSearchResult[];
	sort: TorrentSearchSort;
	error: string | null;
	addingTorrents: Set<string>;
}

const initialState: TorrentSearchState = {
	query: '',
	category: TorrentCategory.All,
	searching: false,
	results: [],
	sort: { field: 'seeders', direction: 'desc' },
	error: null,
	addingTorrents: new Set()
};

class TorrentSearchService {
	public state: Writable<TorrentSearchState> = writable(initialState);

	async search(query: string, category: TorrentCategory = TorrentCategory.All): Promise<void> {
		if (!browser || !query.trim()) return;

		this.state.update((s) => ({
			...s,
			query: query.trim(),
			category,
			searching: true,
			error: null,
			results: []
		}));

		try {
			const params = new URLSearchParams({
				q: query.trim(),
				cat: category
			});
			const response = await fetch(apiUrl(`/api/torrent/search?${params}`));

			if (!response.ok) {
				const body = await response.json().catch(() => ({}));
				throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
			}

			const results: TorrentSearchResult[] = (await response.json()).map(
				(r: TorrentSearchResult) => ({
					...r,
					uploadedAt: new Date(r.uploadedAt)
				})
			);

			this.state.update((s) => ({
				...s,
				searching: false,
				results
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

	toggleSort(field: TorrentSearchSort['field']): void {
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

export const torrentSearchService = new TorrentSearchService();
