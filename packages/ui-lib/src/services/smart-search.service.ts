import { writable } from 'svelte/store';
import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
import type {
	SmartSearchState,
	SmartSearchSelection,
	SmartSearchMovieSelection,
	SmartSearchTvSelection,
	SmartSearchTorrentResult,
	SmartSearchMode,
	SmartSearchMediaType,
	SmartSearchMediaConfig,
	SmartSearchAllConfigs,
	TvSmartSearchResults,
	TvSeasonMeta,
	TvFetchedCandidates,
	MusicSmartSearchResults
} from 'ui-lib/types/smart-search.type';
import type { TorrentSearchResult } from 'addons/torrent-search-thepiratebay/types';
import type { CatalogItem } from 'ui-lib/types/catalog.type';
import { formatAuthors } from 'ui-lib/types/catalog.type';
import { parseTorrentName } from 'addons/torrent-search-thepiratebay/parse-torrent-name';
import { queueService } from 'ui-lib/services/queue.service';

const defaultConfigs: SmartSearchAllConfigs = {
	movies: {
		preferredLanguage: 'English',
		preferredQuality: '1080p',
		smartSearchPrompt: ''
	},
	tv: {
		preferredLanguage: 'English',
		preferredQuality: '1080p',
		smartSearchPrompt: ''
	},
	music: {
		preferredQuality: 'FLAC',
		smartSearchPrompt: ''
	},
	games: {
		preferredConsole: '',
		smartSearchPrompt: ''
	},
	books: {
		preferredFormat: 'EPUB',
		smartSearchPrompt: ''
	}
};

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
	fetchedCandidate: null,
	fetchedTvCandidates: null,
	tvResults: null,
	tvSeasonsMeta: null,
	activeTvTab: 'complete',
	musicResults: null,
	activeMusicTab: 'album'
};

function selectionToMediaType(type: SmartSearchSelection['type']): SmartSearchMediaType {
	switch (type) {
		case 'movie':
			return 'movies';
		case 'tv':
			return 'tv';
		case 'music':
			return 'music';
		case 'game':
			return 'games';
		case 'book':
			return 'books';
	}
}

function getSubdir(selection: SmartSearchSelection): string {
	switch (selection.type) {
		case 'movie':
			return 'movies';
		case 'tv':
			return 'tv';
		case 'music':
			return 'music';
		case 'game':
			return 'games';
		case 'book':
			return 'books';
	}
}

/** Map selection type to the libraryType values returned by the API. */
function getLibraryTypes(selectionType: SmartSearchSelection['type']): string[] {
	switch (selectionType) {
		case 'movie':
			return ['movies'];
		case 'tv':
			return ['tv'];
		case 'music':
			return ['audio', 'music'];
		case 'game':
			return ['games'];
		case 'book':
			return ['books', 'document'];
	}
}

class SmartSearchService {
	public store = writable(initialState);
	public configStore = writable<SmartSearchAllConfigs>(defaultConfigs);
	private abortController: AbortController | null = null;
	private configInitialized = false;

	async initializeConfig(): Promise<void> {
		if (this.configInitialized) return;
		this.configInitialized = true;

		// Migrate old localStorage config if present
		this.migrateOldConfig();

		try {
			const res = await fetchRaw('/api/smart-search/settings');
			if (!res.ok) return;
			const data: SmartSearchAllConfigs = await res.json();
			this.configStore.set(data);
		} catch {
			// Use defaults on failure
		}
	}

	private migrateOldConfig(): void {
		if (typeof window === 'undefined') return;
		const old = localStorage.getItem('smart-search-config');
		if (!old) return;

		try {
			const parsed = JSON.parse(old);
			const updates: Record<string, string> = {};
			if (parsed.preferredLanguage) {
				updates['movies.preferredLanguage'] = parsed.preferredLanguage;
				updates['tv.preferredLanguage'] = parsed.preferredLanguage;
			}
			if (parsed.preferredQuality) {
				updates['movies.preferredQuality'] = parsed.preferredQuality;
				updates['tv.preferredQuality'] = parsed.preferredQuality;
			}
			if (parsed.smartSearchPrompt) {
				updates['movies.smartSearchPrompt'] = parsed.smartSearchPrompt;
			}
			if (Object.keys(updates).length > 0) {
				fetchRaw('/api/smart-search/settings', {
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify(updates)
				}).catch(() => {});
			}
			localStorage.removeItem('smart-search-config');
		} catch {
			localStorage.removeItem('smart-search-config');
		}
	}

	async updateConfig(mediaType: SmartSearchMediaType, field: string, value: string): Promise<void> {
		// Optimistic update
		this.configStore.update((c) => ({
			...c,
			[mediaType]: { ...c[mediaType], [field]: value }
		}));

		try {
			await fetchRaw('/api/smart-search/settings', {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ [`${mediaType}.${field}`]: value })
			});
		} catch {
			// best-effort — optimistic update stays
		}
	}

	getConfigForType(type: SmartSearchSelection['type']): SmartSearchMediaConfig {
		let config: SmartSearchAllConfigs = defaultConfigs;
		this.configStore.subscribe((c) => (config = c))();
		return config[selectionToMediaType(type)];
	}

	select(selection: SmartSearchSelection) {
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
			fetchedCandidate: null,
			fetchedTvCandidates: null,
			tvResults: null,
			tvSeasonsMeta: selection.type === 'tv' ? (selection.seasons ?? null) : null,
			activeTvTab: 'complete',
			musicResults: null,
			activeMusicTab: 'album'
		}));
		if (selection.type === 'tv') {
			this.runTvSearches(selection, this.abortController.signal);
		} else if (selection.type === 'music' && selection.musicSearchMode) {
			this.runMusicSearches(selection, this.abortController.signal);
		} else {
			this.runSearches(selection, this.abortController.signal);
		}
		this.createPendingItem(selection);

		if (selection.mode === 'fetch') {
			if (selection.type === 'tv') {
				this.autoPickTvOnSearchComplete();
			} else {
				this.autoPickOnSearchComplete();
			}
		}
	}

	private autoPickOnSearchComplete() {
		let started = false;
		const unsubscribe = this.store.subscribe((state) => {
			if (state.searching) started = true;
			if (started && !state.searching) {
				unsubscribe();
				if (state.searchError || state.searchResults.length === 0) return;
				const best = this.pickBestFromList(state.searchResults);
				if (best) this.setFetchedCandidate(best);
			}
		});
	}

	private autoPickTvOnSearchComplete() {
		let started = false;
		const unsubscribe = this.store.subscribe((state) => {
			if (state.searching) started = true;
			if (started && !state.searching) {
				unsubscribe();
				if (state.searchError) return;
				const candidates = this.pickBestTvCandidates();
				this.setFetchedTvCandidates(candidates);
			}
		});
	}

	private pickBestTvCandidates(): TvFetchedCandidates {
		const tvResults = this.getState().tvResults;
		if (!tvResults) return { complete: null, seasons: {} };
		const complete = this.pickBestFromList(tvResults.complete);
		const seasons: Record<number, SmartSearchTorrentResult | null> = {};
		for (const [snStr, packs] of Object.entries(tvResults.seasons)) {
			const sn = Number(snStr);
			seasons[sn] = this.pickBestFromList(packs.seasonPacks);
		}
		return { complete, seasons };
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
			fetchedCandidate: null,
			fetchedTvCandidates: null,
			tvResults: null,
			tvSeasonsMeta: null,
			activeTvTab: 'complete',
			musicResults: null,
			activeMusicTab: 'album'
		}));
	}

	private async runSearches(selection: SmartSearchSelection, signal: AbortSignal) {
		const { title, year } = selection;

		let cat: number;
		let queries: string[];

		switch (selection.type) {
			case 'music':
				cat = 100;
				queries = [`${selection.artist} ${title}`];
				break;
			case 'game':
				cat = 400;
				queries = [`${title} ${selection.consoleName}`];
				break;
			case 'book':
				cat = 601;
				queries = [`${title} ${selection.author}`];
				break;
			case 'movie':
				cat = 200;
				queries = [`${title} ${year}`];
				break;
			default:
				cat = 200;
				queries = [`${title} ${year}`];
				break;
		}

		this.store.update((s) => ({ ...s, searching: true, searchError: null }));

		try {
			const seen = new Map<string, SmartSearchTorrentResult>();
			const analyzeHashes = new Set<string>();

			for (const query of queries) {
				if (signal.aborted) return;

				try {
					const res = await fetchRaw(
						`/api/torrent/search?q=${encodeURIComponent(query)}&cat=${cat}`,
						{ signal }
					);
					if (!res.ok) continue;
					const data: TorrentSearchResult[] = await res.json();

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

	private async analyzeResults(selection: SmartSearchSelection, analyzeHashes: Set<string>) {
		const artist =
			selection.type === 'music'
				? selection.artist
				: selection.type === 'book'
					? selection.author
					: undefined;
		const consoleName = selection.type === 'game' ? selection.consoleName : undefined;

		// Step 1: Immediate heuristic analysis (parseTorrentName) — completes synchronously
		this.store.update((s) => {
			const results = s.searchResults.map((r) => {
				if (!analyzeHashes.has(r.infoHash)) return r;
				const analysis = parseTorrentName(
					r.name,
					selection.title,
					selection.year,
					artist,
					consoleName
				);
				return { ...r, analysis };
			});
			return { ...s, searchResults: results, analyzing: false };
		});

		// Step 2: Fire off LLM tasks in background to enhance heuristic results
		this.enhanceWithLlm(selection, analyzeHashes, artist, consoleName);
	}

	private async enhanceWithLlm(
		selection: SmartSearchSelection,
		analyzeHashes: Set<string>,
		artist: string | undefined,
		consoleName: string | undefined
	) {
		queueService.subscribe();

		const config = this.getConfigForType(selection.type);
		const state = this.getState();

		for (const hash of analyzeHashes) {
			const result = state.searchResults.find((r) => r.infoHash === hash);
			if (!result) continue;

			const task = await queueService.createTask('llm:analyze-torrent', {
				torrentName: result.name,
				mediaTitle: selection.title,
				mediaYear: selection.year,
				artist: artist ?? null,
				consoleName: consoleName ?? null,
				promptTemplate: config.smartSearchPrompt ?? ''
			});
			if (!task) continue;

			// Each task resolves independently — update results as they arrive
			queueService
				.waitForTask(task.id)
				.then((completed) => {
					if (completed.status === 'completed' && completed.result) {
						const llmResult = completed.result;
						this.store.update((s) => ({
							...s,
							searchResults: s.searchResults.map((r) => {
								if (r.infoHash !== hash) return r;
								const base = r.analysis ?? {
									quality: '',
									languages: '',
									subs: '',
									relevance: 0,
									reason: '',
									seasonNumber: null,
									episodeNumber: null,
									isCompleteSeries: false,
									isDiscography: false
								};
								return {
									...r,
									analysis: {
										...base,
										quality: (llmResult.quality as string) ?? base.quality,
										languages: (llmResult.languages as string) ?? base.languages,
										subs: (llmResult.subs as string) ?? base.subs,
										relevance: (llmResult.relevance as number) ?? base.relevance,
										reason: (llmResult.reason as string) ?? base.reason
									}
								};
							})
						}));
					}
				})
				.catch(() => {
					// LLM analysis failed; heuristic fallback already applied
				});
		}
	}

	private async runTvSearches(
		selection: SmartSearchSelection & { type: 'tv' },
		signal: AbortSignal
	) {
		const { title } = selection;
		const seasons = selection.seasons ?? [];
		const cat = 200;

		// Build queries: complete series + one per season
		const queries: string[] = [`${title} complete series`];
		for (const s of seasons) {
			if (s.seasonNumber > 0) {
				const sNum = String(s.seasonNumber).padStart(2, '0');
				queries.push(`${title} S${sNum}`);
			}
		}

		this.store.update((s) => ({ ...s, searching: true, searchError: null }));

		try {
			const seen = new Map<string, SmartSearchTorrentResult>();
			const analyzeHashes = new Set<string>();

			for (const query of queries) {
				if (signal.aborted) return;

				try {
					const res = await fetchRaw(
						`/api/torrent/search?q=${encodeURIComponent(query)}&cat=${cat}`,
						{ signal }
					);
					if (!res.ok) continue;
					const data: TorrentSearchResult[] = await res.json();

					const sorted = [...data].sort((a, b) => {
						if (b.seeders !== a.seeders) return b.seeders - a.seeders;
						return b.leechers - a.leechers;
					});
					for (const r of sorted.slice(0, 5)) {
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

			// Analyze all results (not just top 5 per query) for TV since we need season/episode info.
			// Run heuristic analysis + tvResults rebuild synchronously BEFORE flipping `searching: false`,
			// so the auto-pick subscription sees a populated `tvResults`.
			this.analyzeTvResults(selection, analyzeHashes);
			this.store.update((s) => ({ ...s, searching: false }));
		} catch (error) {
			if (signal.aborted) return;
			this.store.update((s) => ({
				...s,
				searching: false,
				searchError: error instanceof Error ? error.message : String(error)
			}));
		}
	}

	private analyzeTvResults(selection: SmartSearchSelection, analyzeHashes: Set<string>) {
		// Heuristic analysis only — completes synchronously.
		// LLM enhancement is intentionally skipped for TV: picks are heuristic-driven and
		// per-scope, so kicking off ~30+ LLM tasks would be wasted compute (the results
		// aren't shown in fetch mode and the picks don't re-rank on LLM completion).
		this.store.update((s) => {
			const results = s.searchResults.map((r) => {
				if (!analyzeHashes.has(r.infoHash)) return r;
				const analysis = parseTorrentName(r.name, selection.title, selection.year);
				return { ...r, analysis };
			});
			return { ...s, searchResults: results, analyzing: false };
		});
		this.rebuildTvResults();
	}

	private rebuildTvResults() {
		this.store.update((s) => {
			const tvResults: TvSmartSearchResults = { complete: [], seasons: {} };

			for (const r of s.searchResults) {
				if (!r.analysis) continue;
				if (r.analysis.isCompleteSeries) {
					tvResults.complete.push(r);
				} else if (r.analysis.seasonNumber != null && r.analysis.episodeNumber != null) {
					const sn = r.analysis.seasonNumber;
					const en = r.analysis.episodeNumber;
					if (!tvResults.seasons[sn]) {
						tvResults.seasons[sn] = { seasonPacks: [], episodes: {} };
					}
					if (!tvResults.seasons[sn].episodes[en]) {
						tvResults.seasons[sn].episodes[en] = [];
					}
					tvResults.seasons[sn].episodes[en].push(r);
				} else if (r.analysis.seasonNumber != null) {
					const sn = r.analysis.seasonNumber;
					if (!tvResults.seasons[sn]) {
						tvResults.seasons[sn] = { seasonPacks: [], episodes: {} };
					}
					tvResults.seasons[sn].seasonPacks.push(r);
				}
			}

			return { ...s, tvResults };
		});
	}

	getBestTvCandidate(): SmartSearchTorrentResult | null {
		const state = this.getState();
		if (!state.tvResults) return null;

		// Strategy: complete series first
		const bestComplete = this.pickBestFromList(state.tvResults.complete);
		if (bestComplete && (bestComplete.analysis?.relevance ?? 0) >= 75) {
			return bestComplete;
		}

		// Fall back to best season pack across all seasons
		const seasonNums = Object.keys(state.tvResults.seasons)
			.map(Number)
			.sort((a, b) => a - b);
		for (const sn of seasonNums) {
			const best = this.pickBestFromList(state.tvResults.seasons[sn].seasonPacks);
			if (best && (best.analysis?.relevance ?? 0) >= 75) {
				return best;
			}
		}

		// Fall back to any complete series even with lower relevance
		if (bestComplete) return bestComplete;

		return null;
	}

	private pickBestFromList(results: SmartSearchTorrentResult[]): SmartSearchTorrentResult | null {
		if (results.length === 0) return null;
		return results.reduce((best, r) => {
			const bestScore = best.analysis?.relevance ?? 0;
			const rScore = r.analysis?.relevance ?? 0;
			if (rScore > bestScore) return r;
			if (rScore === bestScore && r.seeders > best.seeders) return r;
			return best;
		});
	}

	setActiveTvTab(tab: 'complete' | number) {
		this.store.update((s) => ({ ...s, activeTvTab: tab }));
	}

	// --- Music smart search ---

	private async runMusicSearches(
		selection: SmartSearchSelection & { type: 'music' },
		signal: AbortSignal
	) {
		const { artist, title } = selection;
		const cat = 100;

		const queries: string[] = [];
		queries.push(`${artist} ${title}`);
		queries.push(`${artist} discography`);

		this.store.update((s) => ({ ...s, searching: true, searchError: null }));

		try {
			const seen = new Map<string, SmartSearchTorrentResult>();
			const analyzeHashes = new Set<string>();

			for (const query of queries) {
				if (signal.aborted) return;

				try {
					const res = await fetchRaw(
						`/api/torrent/search?q=${encodeURIComponent(query)}&cat=${cat}`,
						{ signal }
					);
					if (!res.ok) continue;
					const data: TorrentSearchResult[] = await res.json();

					const sorted = [...data].sort((a, b) => {
						if (b.seeders !== a.seeders) return b.seeders - a.seeders;
						return b.leechers - a.leechers;
					});
					for (const r of sorted.slice(0, 5)) {
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

			this.analyzeMusicResults(selection, analyzeHashes);
		} catch (error) {
			if (signal.aborted) return;
			this.store.update((s) => ({
				...s,
				searching: false,
				searchError: error instanceof Error ? error.message : String(error)
			}));
		}
	}

	private async analyzeMusicResults(
		selection: SmartSearchSelection & { type: 'music' },
		analyzeHashes: Set<string>
	) {
		this.store.update((s) => {
			const results = s.searchResults.map((r) => {
				if (!analyzeHashes.has(r.infoHash)) return r;
				const analysis = parseTorrentName(
					r.name,
					selection.title,
					selection.year,
					selection.artist
				);
				return { ...r, analysis };
			});
			return { ...s, searchResults: results, analyzing: false };
		});

		this.rebuildMusicResults();

		this.enhanceWithLlm(selection, analyzeHashes, selection.artist, undefined);
	}

	private rebuildMusicResults() {
		this.store.update((s) => {
			const musicResults: MusicSmartSearchResults = { album: [], discography: [] };
			for (const r of s.searchResults) {
				if (!r.analysis) continue;
				if (r.analysis.isDiscography) {
					musicResults.discography.push(r);
				} else {
					musicResults.album.push(r);
				}
			}
			return { ...s, musicResults };
		});
	}

	setActiveMusicTab(tab: 'album' | 'discography') {
		this.store.update((s) => ({ ...s, activeMusicTab: tab }));
	}

	getBestMusicCandidate(): SmartSearchTorrentResult | null {
		const state = this.getState();
		if (!state.musicResults) return null;

		const bestAlbum = this.pickBestFromList(state.musicResults.album);
		if (bestAlbum && (bestAlbum.analysis?.relevance ?? 0) >= 75) {
			return bestAlbum;
		}

		const bestDiscography = this.pickBestFromList(state.musicResults.discography);
		if (bestDiscography && (bestDiscography.analysis?.relevance ?? 0) >= 75) {
			return bestDiscography;
		}

		if (bestAlbum) return bestAlbum;

		return bestDiscography ?? null;
	}

	async checkMusicFetchCache(
		musicbrainzId: string
	): Promise<Array<{ scope: string; candidate: SmartSearchTorrentResult }> | null> {
		try {
			const res = await fetchRaw(
				`/api/catalog/fetch-cache-by-source?source=musicbrainz&sourceId=${musicbrainzId}&kind=artist`
			);
			if (!res.ok) return null;
			const rows: Array<{ scope: string; candidateJson: string }> = await res.json();
			if (rows.length === 0) return null;
			return rows.map((row) => {
				const candidate = JSON.parse(row.candidateJson) as SmartSearchTorrentResult;
				candidate.uploadedAt = new Date(candidate.uploadedAt);
				return { scope: row.scope, candidate };
			});
		} catch {
			return null;
		}
	}

	async saveMusicFetchCache(
		musicbrainzId: string,
		scope: string,
		candidate: SmartSearchTorrentResult
	): Promise<void> {
		try {
			await fetchRaw('/api/catalog/fetch-cache-by-source', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					source: 'musicbrainz',
					sourceId: musicbrainzId,
					kind: 'artist',
					scope,
					scopeKey: '',
					candidate
				})
			});
		} catch {
			// best-effort
		}
	}

	async checkTvFetchCache(tmdbId: number): Promise<Array<{
		scope: string;
		seasonNumber: number | null;
		episodeNumber: number | null;
		candidate: SmartSearchTorrentResult;
	}> | null> {
		try {
			const res = await fetchRaw(
				`/api/catalog/fetch-cache-by-source?source=tmdb&sourceId=${tmdbId}&kind=tv_show`
			);
			if (!res.ok) return null;
			const rows: Array<{ scope: string; scopeKey: string; candidateJson: string }> =
				await res.json();
			if (rows.length === 0) return null;
			return rows.map((row) => {
				const candidate = JSON.parse(row.candidateJson) as SmartSearchTorrentResult;
				candidate.uploadedAt = new Date(candidate.uploadedAt);
				let seasonNumber: number | null = null;
				let episodeNumber: number | null = null;
				if (row.scopeKey) {
					const parts = row.scopeKey.split(':');
					seasonNumber = parts[0] ? Number(parts[0]) : null;
					episodeNumber = parts[1] ? Number(parts[1]) : null;
				}
				return { scope: row.scope, seasonNumber, episodeNumber, candidate };
			});
		} catch {
			return null;
		}
	}

	async clearTvFetchCache(tmdbId: number): Promise<void> {
		try {
			await fetchRaw(
				`/api/catalog/fetch-cache-by-source?source=tmdb&sourceId=${tmdbId}&kind=tv_show`,
				{ method: 'DELETE' }
			);
		} catch {
			// best-effort
		}
	}

	async saveTvFetchCache(
		tmdbId: number,
		scope: string,
		seasonNumber: number | null,
		episodeNumber: number | null,
		candidate: SmartSearchTorrentResult
	): Promise<void> {
		let scopeKey: string;
		if (scope === 'complete') {
			scopeKey = '';
		} else if (scope === 'season') {
			scopeKey = String(seasonNumber);
		} else {
			scopeKey = `${seasonNumber}:${episodeNumber}`;
		}
		try {
			await fetchRaw('/api/catalog/fetch-cache-by-source', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					source: 'tmdb',
					sourceId: String(tmdbId),
					kind: 'tv_show',
					scope,
					scopeKey,
					candidate
				})
			});
		} catch {
			// best-effort
		}
	}

	setFetchedCandidate(candidate: SmartSearchTorrentResult) {
		this.store.update((s) => ({ ...s, fetchedCandidate: candidate }));
	}

	setFetchedTvCandidates(candidates: TvFetchedCandidates) {
		const seasonValues = Object.entries(candidates.seasons)
			.sort(([a], [b]) => Number(a) - Number(b))
			.map(([, c]) => c)
			.filter((c): c is SmartSearchTorrentResult => c !== null);
		const primary = candidates.complete ?? seasonValues[0] ?? null;
		this.store.update((s) => ({
			...s,
			fetchedTvCandidates: candidates,
			fetchedCandidate: primary
		}));
	}

	async selectAndWaitForBest(
		selection: SmartSearchMovieSelection | SmartSearchTvSelection
	): Promise<SmartSearchTorrentResult | null> {
		const cached = await this.checkFetchCache(selection.tmdbId);
		if (cached) return cached;

		const best = await new Promise<SmartSearchTorrentResult | null>((resolve) => {
			let started = false;
			this.select(selection);

			const unsubscribe = this.store.subscribe((state) => {
				if (state.searching) started = true;
				if (started && !state.searching) {
					unsubscribe();
					if (state.searchError || state.searchResults.length === 0) {
						resolve(null);
						return;
					}
					resolve(this.pickBestFromList(state.searchResults));
				}
			});
		});

		if (best) {
			this.setFetchedCandidate(best);
			const mediaType = selection.type === 'tv' ? 'tv' : 'movie';
			this.saveFetchCache(selection.tmdbId, mediaType, best);
			await this.startDownload(best);
		}

		return best;
	}

	setSelection(selection: SmartSearchSelection) {
		this.store.update((s) => ({ ...s, selection }));
	}

	async ensurePendingItem(selection: SmartSearchSelection): Promise<void> {
		const state = this.getState();
		if (state.pendingItemId && state.pendingLibraryId) return;
		await this.createPendingItem(selection);
	}

	getFetchedCandidate(): SmartSearchTorrentResult | null {
		return this.getState().fetchedCandidate;
	}

	async checkFetchCache(tmdbId: number): Promise<SmartSearchTorrentResult | null> {
		try {
			const res = await fetchRaw(
				`/api/catalog/fetch-cache-by-source?source=tmdb&sourceId=${tmdbId}&kind=movie&scope=default&scopeKey=`
			);
			if (!res.ok) return null;
			const data = await res.json();
			const candidate = JSON.parse(data.candidateJson) as SmartSearchTorrentResult;
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
			console.log('[fetch-cache] saving movie cache', { tmdbId, mediaType, name: candidate.name });
			const res = await fetchRaw('/api/catalog/fetch-cache-by-source', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					source: 'tmdb',
					sourceId: String(tmdbId),
					kind: mediaType,
					scope: 'default',
					scopeKey: '',
					candidate
				})
			});
			console.log('[fetch-cache] save response', res.status, res.statusText);
		} catch (err) {
			console.error('[fetch-cache] save failed', err);
		}
	}

	async checkBookFetchCache(openlibraryKey: string): Promise<SmartSearchTorrentResult | null> {
		try {
			const res = await fetchRaw(
				`/api/catalog/fetch-cache-by-source?source=openlibrary&sourceId=${encodeURIComponent(openlibraryKey)}&kind=book&scope=default&scopeKey=`
			);
			if (!res.ok) return null;
			const data = await res.json();
			const candidate = JSON.parse(data.candidateJson) as SmartSearchTorrentResult;
			candidate.uploadedAt = new Date(candidate.uploadedAt);
			return candidate;
		} catch {
			return null;
		}
	}

	async saveBookFetchCache(
		openlibraryKey: string,
		candidate: SmartSearchTorrentResult
	): Promise<void> {
		try {
			await fetchRaw('/api/catalog/fetch-cache-by-source', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					source: 'openlibrary',
					sourceId: openlibraryKey,
					kind: 'book',
					scope: 'default',
					scopeKey: '',
					candidate
				})
			});
		} catch {
			// best-effort
		}
	}

	async checkGameFetchCache(retroachievementsId: number): Promise<SmartSearchTorrentResult | null> {
		try {
			const res = await fetchRaw(
				`/api/catalog/fetch-cache-by-source?source=retroachievements&sourceId=${retroachievementsId}&kind=game&scope=default&scopeKey=`
			);
			if (!res.ok) return null;
			const data = await res.json();
			const candidate = JSON.parse(data.candidateJson) as SmartSearchTorrentResult;
			candidate.uploadedAt = new Date(candidate.uploadedAt);
			return candidate;
		} catch {
			return null;
		}
	}

	async saveGameFetchCache(
		retroachievementsId: number,
		candidate: SmartSearchTorrentResult
	): Promise<void> {
		try {
			await fetchRaw('/api/catalog/fetch-cache-by-source', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					source: 'retroachievements',
					sourceId: String(retroachievementsId),
					kind: 'game',
					scope: 'default',
					scopeKey: '',
					candidate
				})
			});
		} catch {
			// best-effort
		}
	}

	async startDownload(candidate: SmartSearchTorrentResult): Promise<string | null> {
		const selection = this.getSelection();
		if (!selection) return null;

		try {
			const downloadPath = await this.resolveDownloadPath(selection);
			if (!downloadPath) return null;

			const res = await fetchRaw('/api/torrent/torrents', {
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
			const downloadPath = await this.resolveDownloadPath(selection);
			if (!downloadPath) return null;

			const res = await fetchRaw('/api/torrent/torrents', {
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
			await fetchRaw(
				`/api/libraries/${state.pendingLibraryId}/items/${state.pendingItemId}/torrent`,
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

	/** Resolve the correct download path for the given selection by looking up the library. */
	private async resolveDownloadPath(selection: SmartSearchSelection): Promise<string | null> {
		try {
			const libRes = await fetchRaw('/api/libraries');
			if (!libRes.ok) return null;
			const libraries: Array<{ id: string; path: string; libraryType: string }> =
				await libRes.json();
			const types = getLibraryTypes(selection.type);
			const library = libraries.find((l) => types.includes(l.libraryType));
			if (library) return library.path;

			// Fallback: torrent config path + subdir
			const configRes = await fetchRaw('/api/torrent/config');
			if (!configRes.ok) return null;
			const config = await configRes.json();
			const basePath: string = config.downloadPath ?? '';
			return basePath || null;
		} catch {
			return null;
		}
	}

	private async createPendingItem(selection: SmartSearchSelection) {
		// If the item already exists in the library, skip creation
		if (
			(selection.type === 'movie' || selection.type === 'tv') &&
			selection.existingItemId &&
			selection.existingLibraryId
		) {
			this.store.update((s) => ({
				...s,
				pendingItemId: selection.existingItemId!,
				pendingLibraryId: selection.existingLibraryId!
			}));
			return;
		}

		try {
			const libRes = await fetchRaw('/api/libraries');
			if (!libRes.ok) return;
			const libraries: Array<{ id: string; path: string; libraryType: string }> =
				await libRes.json();

			const types = getLibraryTypes(selection.type);
			let library = libraries.find((l) => types.includes(l.libraryType));
			if (!library) {
				// Derive the library path from an existing library's parent directory
				const anyLib = libraries[0];
				if (!anyLib) return;
				const parentPath = anyLib.path.replace(/\/[^/]+\/?$/, '');
				const subdir = getSubdir(selection);
				const newPath = `${parentPath}/${subdir}`;

				let libName: string;
				switch (selection.type) {
					case 'music':
						libName = 'Music';
						break;
					case 'movie':
						libName = 'Movies';
						break;
					case 'tv':
						libName = 'TV Shows';
						break;
					case 'game':
						libName = 'Games';
						break;
					case 'book':
						libName = 'Books';
						break;
				}
				const createRes = await fetchRaw('/api/libraries', {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({
						name: libName,
						path: newPath,
						libraryType: subdir
					})
				});
				if (!createRes.ok) return;
				library = await createRes.json();
			}
			if (!library) return;

			let pendingName: string;
			let mediaType: string;
			let categoryId: string;

			switch (selection.type) {
				case 'music':
					pendingName = `${selection.artist} - ${selection.title}`;
					mediaType = 'audio';
					categoryId = 'audio-uncategorized';
					break;
				case 'game':
					pendingName = `${selection.title} (${selection.consoleName})`;
					mediaType = 'video';
					categoryId = 'games';
					break;
				case 'book':
					pendingName = `${selection.author} - ${selection.title}`;
					mediaType = 'document';
					categoryId = 'books';
					break;
				default:
					pendingName = selection.title;
					mediaType = 'video';
					categoryId = selection.type === 'movie' ? 'movies' : 'tv';
					break;
			}

			const pendingPath = `${library.path}/${pendingName}`;

			const itemBody: Record<string, unknown> = {
				name: pendingName,
				path: pendingPath,
				mediaType,
				categoryId
			};
			if (selection.type === 'movie' || selection.type === 'tv') {
				itemBody.tmdbId = selection.tmdbId;
			}

			const itemRes = await fetchRaw(`/api/libraries/${library.id}/items`, {
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

				if (selection.type === 'music') {
					try {
						await fetchRaw(`/api/libraries/${library!.id}/items/${item.id}/musicbrainz`, {
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

	selectFromCatalog(item: CatalogItem, mode: SmartSearchMode = 'fetch') {
		let selection: SmartSearchSelection;
		switch (item.kind) {
			case 'movie':
				selection = {
					type: 'movie',
					tmdbId: item.metadata.tmdbId,
					title: item.title,
					year: item.year ?? '',
					mode
				};
				break;
			case 'tv_show':
				selection = {
					type: 'tv',
					tmdbId: item.metadata.tmdbId,
					title: item.title,
					year: item.year ?? '',
					mode,
					seasons: item.metadata.seasons.map((s) => ({
						seasonNumber: s.seasonNumber,
						name: s.name,
						episodeCount: s.episodeCount,
						episodes: []
					}))
				};
				break;
			case 'album':
				selection = {
					type: 'music',
					musicbrainzId: item.metadata.musicbrainzId,
					title: item.title,
					year: item.year ?? '',
					artist: formatAuthors(item.metadata.authors, 'artist'),
					mode
				};
				break;
			case 'game':
				selection = {
					type: 'game',
					retroachievementsId: item.metadata.retroachievementsId,
					title: item.title,
					year: item.year ?? '',
					consoleName: item.metadata.consoleName,
					mode
				};
				break;
			case 'book':
				selection = {
					type: 'book',
					openlibraryKey: item.metadata.openlibraryKey,
					title: item.title,
					year: item.year ?? '',
					author: item.metadata.authors[0]?.name ?? '',
					mode
				};
				break;
			default:
				return;
		}
		this.select(selection);
	}
}

export const smartSearchService = new SmartSearchService();
