import { writable } from 'svelte/store';
import { apiUrl } from 'frontend/lib/api-base';
import type {
	SmartSearchState,
	SmartSearchSelection,
	SmartSearchTorrentResult
} from 'frontend/types/smart-search.type';
import type { TorrentSearchResult } from 'addons/torrent-search-thepiratebay/types';
import { parseTorrentName } from 'frontend/utils/torrent-search/parse-torrent-name';

const initialState: SmartSearchState = {
	selection: null,
	visible: false,
	searching: false,
	analyzing: false,
	searchResults: [],
	searchError: null
};

class SmartSearchService {
	public store = writable(initialState);
	private abortController: AbortController | null = null;

	select(selection: SmartSearchSelection) {
		// Cancel any in-flight searches/analysis
		if (this.abortController) {
			this.abortController.abort();
		}
		this.abortController = new AbortController();

		this.store.update((s) => ({
			...s,
			selection,
			visible: true,
			searching: false,
			analyzing: false,
			searchResults: [],
			searchError: null
		}));
		this.runSearches(selection, this.abortController.signal);
	}

	clear() {
		if (this.abortController) {
			this.abortController.abort();
			this.abortController = null;
		}
		this.store.update((s) => ({
			...s,
			selection: null,
			searchResults: [],
			searchError: null,
			analyzing: false
		}));
	}

	private async runSearches(selection: SmartSearchSelection, signal: AbortSignal) {
		const { title, year } = selection;
		const queries = [
			title,
			`${title} ${year}`
		];

		this.store.update((s) => ({ ...s, searching: true, searchError: null }));

		try {
			const seen = new Map<string, SmartSearchTorrentResult>();
			const analyzeHashes = new Set<string>();

			for (const query of queries) {
				if (signal.aborted) return;

				try {
					const url = apiUrl(
						`/api/torrent/search?q=${encodeURIComponent(query)}&cat=200`
					);
					const res = await fetch(url, { signal });
					if (!res.ok) continue;
					const data: TorrentSearchResult[] = await res.json();

					// Pick top 5 from this query by SE then LE
					const sorted = [...data].sort((a, b) => {
						if (b.seeders !== a.seeders) return b.seeders - a.seeders;
						return b.leechers - a.leechers;
					});
					const top = sorted.slice(0, 5);
					for (const r of top) {
						analyzeHashes.add(r.infoHash);
					}

					for (const r of data) {
						const existing = seen.get(r.infoHash);
						if (existing) {
							existing.searchQueries.push(query);
						} else {
							seen.set(r.infoHash, {
								...r,
								uploadedAt: new Date(r.uploadedAt),
								searchQueries: [query],
								analysis: null,
								analyzing: false
							});
						}
					}
				} catch (e) {
					if (signal.aborted) return;
				}

				const current = [...seen.values()];
				this.store.update((s) => ({ ...s, searchResults: current }));
			}

			if (signal.aborted) return;
			this.store.update((s) => ({ ...s, searching: false }));

			this.analyzeResults(selection, analyzeHashes);
		} catch (error) {
			if (signal.aborted) return;
			this.store.update((s) => ({
				...s,
				searching: false,
				searchError: error instanceof Error ? error.message : String(error)
			}));
		}
	}

	private analyzeResults(selection: SmartSearchSelection, analyzeHashes: Set<string>) {
		this.store.update((s) => {
			const results = s.searchResults.map((r) => {
				if (!analyzeHashes.has(r.infoHash)) return r;
				const analysis = parseTorrentName(r.name, selection.title, selection.year);
				return { ...r, analysis, analyzing: false };
			});
			return { ...s, searchResults: results, analyzing: false };
		});
	}
}

export const smartSearchService = new SmartSearchService();
