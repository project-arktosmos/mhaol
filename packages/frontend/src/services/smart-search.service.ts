import { writable } from 'svelte/store';
import { apiUrl } from 'frontend/lib/api-base';
import type { SmartSearchState, SmartSearchSelection } from 'frontend/types/smart-search.type';
import type { TorrentSearchResult } from 'addons/torrent-search-thepiratebay/types';

const initialState: SmartSearchState = {
	selection: null,
	visible: true,
	searching: false,
	searchResults: [],
	searchError: null
};

class SmartSearchService {
	public store = writable(initialState);

	select(selection: SmartSearchSelection) {
		this.store.update((s) => ({
			...s,
			selection,
			visible: true,
			searchResults: [],
			searchError: null
		}));
		this.runSearches(selection);
	}

	clear() {
		this.store.update((s) => ({ ...s, selection: null, searchResults: [], searchError: null }));
	}

	private async runSearches(selection: SmartSearchSelection) {
		const { title, year, type } = selection;
		const typeLabel = type === 'movie' ? 'movie' : 'tv show';
		const queries = [
			title,
			`${title} ${year}`,
			`${title} ${typeLabel}`,
			`${title} ${year} ${typeLabel}`
		];

		this.store.update((s) => ({ ...s, searching: true, searchError: null }));

		try {
			const results = await Promise.all(
				queries.map(async (query) => {
					try {
						const url = apiUrl(
							`/api/torrent/search?q=${encodeURIComponent(query)}&cat=200`
						);
						const res = await fetch(url);
						if (!res.ok) return [];
						const data: TorrentSearchResult[] = await res.json();
						return data.map((r) => ({
							...r,
							uploadedAt: new Date(r.uploadedAt)
						}));
					} catch {
						return [];
					}
				})
			);

			const seen = new Set<string>();
			const deduplicated: TorrentSearchResult[] = [];
			for (const batch of results) {
				for (const result of batch) {
					if (!seen.has(result.infoHash)) {
						seen.add(result.infoHash);
						deduplicated.push(result);
					}
				}
			}

			deduplicated.sort((a, b) => b.seeders - a.seeders);

			this.store.update((s) => ({
				...s,
				searching: false,
				searchResults: deduplicated
			}));
		} catch (error) {
			this.store.update((s) => ({
				...s,
				searching: false,
				searchError: error instanceof Error ? error.message : String(error)
			}));
		}
	}
}

export const smartSearchService = new SmartSearchService();
