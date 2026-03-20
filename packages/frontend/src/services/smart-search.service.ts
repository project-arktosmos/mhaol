import { writable } from 'svelte/store';
import { apiUrl } from 'frontend/lib/api-base';
import type {
	SmartSearchState,
	SmartSearchSelection,
	SmartSearchTorrentResult,
	SmartSearchMode
} from 'frontend/types/smart-search.type';
import type { TorrentSearchResult } from 'addons/torrent-search-thepiratebay/types';
import { parseTorrentName } from 'frontend/utils/torrent-search/parse-torrent-name';

const initialState: SmartSearchState = {
	selection: null,
	visible: false,
	searching: false,
	analyzing: false,
	searchResults: [],
	searchError: null,
	streamingHash: null,
	streamingProgress: 0,
	pendingItemId: null,
	pendingLibraryId: null
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
			searchError: null,
			pendingItemId: null,
			pendingLibraryId: null
		}));
		this.runSearches(selection, this.abortController.signal);
		this.createPendingItem(selection);
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
			analyzing: false,
			pendingItemId: null,
			pendingLibraryId: null
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

	async startStream(candidate: SmartSearchTorrentResult): Promise<string | null> {
		const selection = this.getSelection();
		if (!selection) return null;

		try {
			const configRes = await fetch(apiUrl('/api/torrent/config'));
			if (!configRes.ok) return null;
			const config = await configRes.json();
			const basePath: string = config.downloadPath ?? '';
			if (!basePath) return null;

			const subdir = selection.type === 'movie' ? 'movies' : 'tv';
			const downloadPath = `${basePath}/${subdir}`;

			const res = await fetch(apiUrl('/api/torrent/torrents'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					source: candidate.magnetLink,
					downloadPath
				})
			});
			if (!res.ok) return null;

			const torrentInfo = await res.json();
			const infoHash: string = torrentInfo.infoHash ?? candidate.infoHash;
			const outputPath: string = torrentInfo.outputPath ?? downloadPath;

			await this.updateItemWithTorrent(infoHash, outputPath, 'stream');

			this.store.update((s) => ({ ...s, streamingHash: infoHash, streamingProgress: 0 }));
			return infoHash;
		} catch {
			return null;
		}
	}

	async updateItemWithTorrent(infoHash: string, outputPath: string, mode: SmartSearchMode): Promise<void> {
		const state = this.getState();
		if (!state.pendingItemId || !state.pendingLibraryId) return;

		try {
			await fetch(apiUrl(`/api/libraries/${state.pendingLibraryId}/items/${state.pendingItemId}/torrent`), {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ infoHash, outputPath, mode })
			});
		} catch {
			// best-effort
		}
	}

	updateStreamingProgress(progress: number) {
		this.store.update((s) => ({ ...s, streamingProgress: progress }));
	}

	hide() {
		this.store.update((s) => ({ ...s, visible: false }));
	}

	clearStreaming() {
		this.store.update((s) => ({ ...s, streamingHash: null, streamingProgress: 0 }));
	}

	private getState(): SmartSearchState {
		let state: SmartSearchState = initialState;
		this.store.subscribe((s) => (state = s))();
		return state;
	}

	private getSelection(): SmartSearchSelection | null {
		return this.getState().selection;
	}

	private async createPendingItem(selection: SmartSearchSelection) {
		try {
			const configRes = await fetch(apiUrl('/api/torrent/config'));
			if (!configRes.ok) return;
			const config = await configRes.json();
			const basePath: string = config.downloadPath ?? '';
			if (!basePath) return;

			const subdir = selection.type === 'movie' ? 'movies' : 'tv';
			const targetPath = `${basePath}/${subdir}`;

			const libRes = await fetch(apiUrl('/api/libraries'));
			if (!libRes.ok) return;
			const libraries: Array<{ id: string; path: string }> = await libRes.json();
			let library = libraries.find((l) => l.path === targetPath);

			if (!library) {
				const createRes = await fetch(apiUrl('/api/libraries'), {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({
						name: selection.type === 'movie' ? 'Movies' : 'TV Shows',
						path: targetPath,
						libraryType: subdir
					})
				});
				if (!createRes.ok) return;
				library = await createRes.json();
			}
			if (!library) return;

			const pendingPath = `${targetPath}/${selection.title}`;
			const itemRes = await fetch(apiUrl(`/api/libraries/${library.id}/items`), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					name: selection.title,
					path: pendingPath,
					mediaType: 'video',
					categoryId: subdir === 'movies' ? 'movies' : 'tv',
					tmdbId: selection.tmdbId
				})
			});
			if (itemRes.ok) {
				const item = await itemRes.json();
				this.store.update((s) => ({
					...s,
					pendingItemId: item.id,
					pendingLibraryId: library!.id
				}));
			}
		} catch {
			// best-effort
		}
	}
}

export const smartSearchService = new SmartSearchService();
