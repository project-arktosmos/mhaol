import { writable, type Readable } from 'svelte/store';
import { fetchRaw } from '$transport/fetch-helpers';

interface FetchCacheSummaryEntry {
	source: string;
	sourceId: string;
	scope: string;
	name: string;
}

interface FetchCacheHashEntry {
	source: string;
	sourceId: string;
	infoHash: string;
}

export interface FetchCacheState {
	cachedIds: Set<number>;
	hashes: Map<number, string>;
	summaries: Map<number, string>;
}

const initialState: FetchCacheState = {
	cachedIds: new Set(),
	hashes: new Map(),
	summaries: new Map()
};

class FetchCacheService {
	private _state = writable<FetchCacheState>(initialState);
	private source: string | null = null;

	get state(): Readable<FetchCacheState> {
		return this._state;
	}

	async load(source: string): Promise<void> {
		this.source = source;
		await Promise.all([this.loadSummaries(source), this.loadHashes(source)]);
	}

	async refresh(): Promise<void> {
		if (this.source) await this.load(this.source);
	}

	private async loadSummaries(source: string): Promise<void> {
		try {
			const res = await fetchRaw('/api/catalog/fetch-cache/summaries');
			if (!res.ok) return;
			const entries: FetchCacheSummaryEntry[] = await res.json();
			const filtered = entries.filter((e) => e.source === source);
			this._state.update((s) => ({
				...s,
				cachedIds: new Set(filtered.map((e) => Number(e.sourceId))),
				summaries: new Map(filtered.map((e) => [Number(e.sourceId), e.name]))
			}));
		} catch {
			/* best-effort */
		}
	}

	private async loadHashes(source: string): Promise<void> {
		try {
			const res = await fetchRaw('/api/catalog/fetch-cache/hashes');
			if (!res.ok) return;
			const entries: FetchCacheHashEntry[] = await res.json();
			const filtered = entries.filter((e) => e.source === source);
			this._state.update((s) => ({
				...s,
				hashes: new Map(filtered.map((e) => [Number(e.sourceId), e.infoHash]))
			}));
		} catch {
			/* best-effort */
		}
	}
}

export const fetchCacheService = new FetchCacheService();
