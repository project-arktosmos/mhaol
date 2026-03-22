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
	pendingLibraryId: null,
	downloadedHash: null,
	fetchedCandidate: null
};

function getSubdir(selection: SmartSearchSelection): string {
	if (selection.type === 'music') return 'music';
	return selection.type === 'movie' ? 'movies' : 'tv';
}

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
			visible: false,
			searching: false,
			analyzing: false,
			searchResults: [],
			searchError: null,
			pendingItemId: null,
			pendingLibraryId: null,
			downloadedHash: null,
			fetchedCandidate: null
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
			pendingLibraryId: null,
			fetchedCandidate: null
		}));
	}

	private async runSearches(selection: SmartSearchSelection, signal: AbortSignal) {
		const { title, year } = selection;

		const cat = selection.type === 'music' ? 100 : 200;
		const queries =
			selection.type === 'music'
				? [`${selection.artist} ${title}`, selection.artist]
				: [title, `${title} ${year}`];

		this.store.update((s) => ({ ...s, searching: true, searchError: null }));

		try {
			const seen = new Map<string, SmartSearchTorrentResult>();
			const analyzeHashes = new Set<string>();

			for (const query of queries) {
				if (signal.aborted) return;

				try {
					const url = apiUrl(`/api/torrent/search?q=${encodeURIComponent(query)}&cat=${cat}`);
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
				} catch {
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
		const artist = selection.type === 'music' ? selection.artist : undefined;
		this.store.update((s) => {
			const results = s.searchResults.map((r) => {
				if (!analyzeHashes.has(r.infoHash)) return r;
				const analysis = parseTorrentName(r.name, selection.title, selection.year, artist);
				return { ...r, analysis, analyzing: false };
			});
			return { ...s, searchResults: results, analyzing: false };
		});
	}

	setFetchedCandidate(candidate: SmartSearchTorrentResult) {
		this.store.update((s) => ({ ...s, fetchedCandidate: candidate }));
	}

	/** Set the selection without triggering searches (used when restoring from cache). */
	setSelection(selection: SmartSearchSelection) {
		this.store.update((s) => ({ ...s, selection }));
	}

	getFetchedCandidate(): SmartSearchTorrentResult | null {
		return this.getState().fetchedCandidate;
	}

	async checkFetchCache(tmdbId: number): Promise<SmartSearchTorrentResult | null> {
		try {
			const res = await fetch(apiUrl(`/api/torrent/fetch-cache/${tmdbId}`));
			if (!res.ok) return null;
			const data = await res.json();
			const candidate = data.candidate as SmartSearchTorrentResult;
			candidate.uploadedAt = new Date(candidate.uploadedAt);
			return candidate;
		} catch {
			return null;
		}
	}

	async saveFetchCache(
		tmdbId: number,
		mediaType: string,
		candidate: SmartSearchTorrentResult
	): Promise<void> {
		try {
			await fetch(apiUrl('/api/torrent/fetch-cache'), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ tmdbId, mediaType, candidate })
			});
		} catch {
			// best-effort
		}
	}

	async startDownload(candidate: SmartSearchTorrentResult): Promise<string | null> {
		const selection = this.getSelection();
		if (!selection) return null;

		try {
			const configRes = await fetch(apiUrl('/api/torrent/config'));
			if (!configRes.ok) return null;
			const config = await configRes.json();
			const basePath: string = config.downloadPath ?? '';
			if (!basePath) return null;

			const subdir = getSubdir(selection);
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

			await this.updateItemWithTorrent(infoHash, outputPath, 'download');

			this.store.update((s) => ({ ...s, downloadedHash: infoHash }));
			return infoHash;
		} catch {
			return null;
		}
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

			const subdir = getSubdir(selection);
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

	async updateItemWithTorrent(
		infoHash: string,
		outputPath: string,
		mode: SmartSearchMode
	): Promise<void> {
		const state = this.getState();
		if (!state.pendingItemId || !state.pendingLibraryId) return;

		this.store.update((s) => ({ ...s, downloadedHash: infoHash }));

		try {
			await fetch(
				apiUrl(`/api/libraries/${state.pendingLibraryId}/items/${state.pendingItemId}/torrent`),
				{
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ infoHash, outputPath, mode })
				}
			);
		} catch {
			// best-effort
		}
	}

	updateStreamingProgress(progress: number) {
		this.store.update((s) => ({ ...s, streamingProgress: progress }));
	}

	show() {
		this.store.update((s) => ({ ...s, visible: true }));
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
		// If the item already exists in the library, skip creation
		if (selection.type !== 'music' && selection.existingItemId && selection.existingLibraryId) {
			this.store.update((s) => ({
				...s,
				pendingItemId: selection.existingItemId!,
				pendingLibraryId: selection.existingLibraryId!
			}));
			return;
		}

		try {
			const configRes = await fetch(apiUrl('/api/torrent/config'));
			if (!configRes.ok) return;
			const config = await configRes.json();
			const basePath: string = config.downloadPath ?? '';
			if (!basePath) return;

			const subdir = getSubdir(selection);
			const targetPath = `${basePath}/${subdir}`;

			const libRes = await fetch(apiUrl('/api/libraries'));
			if (!libRes.ok) return;
			const libraries: Array<{ id: string; path: string }> = await libRes.json();
			let library = libraries.find((l) => l.path === targetPath);

			if (!library) {
				const libName =
					selection.type === 'music' ? 'Music' : selection.type === 'movie' ? 'Movies' : 'TV Shows';
				const createRes = await fetch(apiUrl('/api/libraries'), {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({
						name: libName,
						path: targetPath,
						libraryType: subdir
					})
				});
				if (!createRes.ok) return;
				library = await createRes.json();
			}
			if (!library) return;

			const pendingName =
				selection.type === 'music' ? `${selection.artist} - ${selection.title}` : selection.title;
			const pendingPath = `${targetPath}/${pendingName}`;
			const mediaType = selection.type === 'music' ? 'audio' : 'video';
			const categoryId =
				selection.type === 'music' ? 'audio-uncategorized' : subdir === 'movies' ? 'movies' : 'tv';

			const itemBody: Record<string, unknown> = {
				name: pendingName,
				path: pendingPath,
				mediaType,
				categoryId
			};
			if (selection.type !== 'music') {
				itemBody.tmdbId = selection.tmdbId;
			}

			const itemRes = await fetch(apiUrl(`/api/libraries/${library.id}/items`), {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(itemBody)
			});
			if (itemRes.ok) {
				const item = await itemRes.json();
				this.store.update((s) => ({
					...s,
					pendingItemId: item.id,
					pendingLibraryId: library!.id
				}));

				// Link MusicBrainz ID after item creation
				if (selection.type === 'music') {
					try {
						await fetch(apiUrl(`/api/libraries/${library!.id}/items/${item.id}/musicbrainz`), {
							method: 'PUT',
							headers: { 'Content-Type': 'application/json' },
							body: JSON.stringify({ musicbrainzId: selection.musicbrainzId })
						});
					} catch {
						// best-effort
					}
				}
			}
		} catch {
			// best-effort
		}
	}
}

export const smartSearchService = new SmartSearchService();
