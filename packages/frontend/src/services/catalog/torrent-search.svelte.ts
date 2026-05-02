import { base } from '$app/paths';
import {
	matchTorrentsForResult,
	parseTorrentName,
	searchTorrents,
	tpbCategoryFor,
	type TorrentResultItem
} from '$lib/search.service';
import { addonKind } from '$lib/firkins.service';

type RawTorrent = Omit<TorrentResultItem, 'parsedTitle' | 'year' | 'quality'>;

interface FirkinTorrentSearchResponse {
	results: RawTorrent[];
	evals?: Record<string, unknown>;
}

async function fetchFirkinTorrentSearch(
	firkinId: string
): Promise<FirkinTorrentSearchResponse | null> {
	try {
		const res = await fetch(`${base}/api/firkins/${encodeURIComponent(firkinId)}/torrent-search`);
		if (!res.ok) return null;
		const body = (await res.json()) as FirkinTorrentSearchResponse | null;
		return body ?? null;
	} catch {
		return null;
	}
}

async function refreshFirkinTorrentSearch(
	firkinId: string,
	addon: string,
	query: string,
	category: string
): Promise<FirkinTorrentSearchResponse | null> {
	try {
		const res = await fetch(`${base}/api/firkins/${encodeURIComponent(firkinId)}/torrent-search`, {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ addon, query, category })
		});
		if (!res.ok) return null;
		return (await res.json()) as FirkinTorrentSearchResponse;
	} catch {
		return null;
	}
}

function evalToRowEval(value: unknown): TorrentRowEval | null {
	if (!value || typeof value !== 'object') return null;
	const v = value as Record<string, unknown>;
	if (v.streamable === true) {
		return {
			kind: 'streamable',
			fileName: typeof v.fileName === 'string' ? v.fileName : '',
			fileSize: typeof v.fileSize === 'number' ? v.fileSize : 0,
			mimeType: typeof v.mimeType === 'string' ? v.mimeType : null
		};
	}
	if (v.streamable === false) {
		return {
			kind: 'not-streamable',
			reason: typeof v.reason === 'string' ? v.reason : 'not streamable'
		};
	}
	return null;
}

export type TorrentSearchStatus = 'idle' | 'searching' | 'done' | 'error';

export interface SearchStackEntry {
	id: number;
	/** Human-readable label (e.g. `"Show"`, `"Season 5"`). */
	label: string;
	/** Actual query sent to the indexer. */
	query: string;
	status: 'searching' | 'done' | 'error';
	/** Result count: total matches for the initial `search()`, or new
	 * additions for an `searchAppend()` call. Set once `status` flips to
	 * `done`. */
	count?: number;
	error?: string;
}

export type TorrentRowEval =
	| { kind: 'pending' }
	| { kind: 'evaluating' }
	| { kind: 'streamable'; fileName: string; fileSize: number; mimeType: string | null }
	| { kind: 'not-streamable'; reason: string }
	| { kind: 'skipped'; reason: string };

export interface TorrentSearchOptions {
	/** When true, runs `/api/torrent/evaluate` for each result so the UI can mark streamability. */
	evaluate?: boolean;
}

interface RunArgs {
	addon: string;
	title: string;
	year: number | null;
	/** When provided, the service hydrates from / persists to the firkin-scoped
	 * backend cache via `/api/firkins/:id/torrent-search`. The response also
	 * carries cached per-magnet probe evals so the streamability column lights
	 * up immediately on revisit instead of replaying every `/api/torrent/evaluate`
	 * round-trip. Omitting this falls back to the legacy local-storage flow. */
	firkinId?: string;
	/** Skip the GET-cache check and POST a fresh search to the backend (which
	 * also re-persists). Used by the **Refresh** button when the user wants
	 * fresh indexer results. */
	forceRefresh?: boolean;
}

export interface TorrentQualityGroup {
	label: string;
	probe: boolean;
	rows: TorrentResultItem[];
}

const QUALITY_GROUPS: {
	label: string;
	probe: boolean;
	matches: (q: string | null) => boolean;
}[] = [
	{ label: '4K', probe: true, matches: (q) => q === '4K' || q === '4K UHD' },
	{ label: '2160p', probe: true, matches: (q) => q === '2160p' },
	{ label: '1080p', probe: true, matches: (q) => q === '1080p' },
	{ label: '720p', probe: true, matches: (q) => q === '720p' },
	{ label: '480p', probe: true, matches: (q) => q === '480p' },
	{ label: '360p', probe: true, matches: (q) => q === '360p' },
	{ label: 'Other', probe: false, matches: () => true }
];

function groupIndex(quality: string | null): number {
	for (let i = 0; i < QUALITY_GROUPS.length; i++) {
		if (QUALITY_GROUPS[i].matches(quality)) return i;
	}
	return QUALITY_GROUPS.length - 1;
}

function groupMatches(matches: TorrentResultItem[]): TorrentQualityGroup[] {
	const buckets = new Map<number, TorrentResultItem[]>();
	for (const t of matches) {
		const idx = groupIndex(t.quality);
		let bucket = buckets.get(idx);
		if (!bucket) {
			bucket = [];
			buckets.set(idx, bucket);
		}
		bucket.push(t);
	}
	for (const list of buckets.values()) {
		list.sort((a, b) => b.seeders - a.seeders || b.leechers - a.leechers);
	}
	const out: TorrentQualityGroup[] = [];
	for (let i = 0; i < QUALITY_GROUPS.length; i++) {
		const rows = buckets.get(i);
		if (rows && rows.length > 0)
			out.push({ label: QUALITY_GROUPS[i].label, probe: QUALITY_GROUPS[i].probe, rows });
	}
	return out;
}

export class TorrentSearch {
	matches = $state<TorrentResultItem[]>([]);
	status = $state<TorrentSearchStatus>('idle');
	error = $state<string | null>(null);
	rowEvals = $state<Record<string, TorrentRowEval>>({});
	groupedMatches = $derived<TorrentQualityGroup[]>(groupMatches(this.matches));
	/// Per-query progress log. Each `search()` resets the log; each
	/// `searchAppend()` pushes one entry. The UI surfaces this so the user
	/// can see when the initial show-name search and each per-season fan-out
	/// search are running, completed, or failed.
	searchStack = $state<SearchStackEntry[]>([]);

	private run = 0;
	private nextEntryId = 0;
	private readonly evaluate: boolean;

	constructor(options: TorrentSearchOptions = {}) {
		this.evaluate = options.evaluate ?? false;
	}

	cancel(): void {
		this.run++;
	}

	private pushStackEntry(label: string, query: string): number {
		const id = ++this.nextEntryId;
		this.searchStack = [...this.searchStack, { id, label, query, status: 'searching' }];
		return id;
	}

	private updateStackEntry(id: number, patch: Partial<SearchStackEntry>): void {
		this.searchStack = this.searchStack.map((e) => (e.id === id ? { ...e, ...patch } : e));
	}

	/// Run a focused query (e.g. `"Show Name S02"`) against the torrent
	/// indexer and **append** any new matches to `matches` — does not reset
	/// the existing list. `matchTitle` is the firkin's title (used by the
	/// fuzzy matcher so season-tagged torrent titles like `Show.Name.S02`
	/// still match the show). Aborts cleanly if a fresh `search()` was
	/// initiated while this call was in flight.
	async searchAppend(
		addon: string,
		query: string,
		matchTitle: string,
		year: number | null,
		label?: string
	): Promise<void> {
		const tokenAtStart = this.run;
		const entryId = this.pushStackEntry(label ?? query, query);
		try {
			const torrents = await searchTorrents(addon, query);
			if (tokenAtStart !== this.run) {
				this.updateStackEntry(entryId, { status: 'error', error: 'cancelled' });
				return;
			}
			const kind = addonKind(addon);
			const isTv = kind === 'tv show';
			const isMovie = kind === 'movie';
			const fresh = matchTorrentsForResult(
				{
					title: matchTitle,
					description: '',
					artists: [],
					images: [],
					files: [],
					year,
					raw: null
				},
				torrents,
				{ skipYearFilter: isTv, excludeTvSeries: isMovie }
			);
			if (tokenAtStart !== this.run) {
				this.updateStackEntry(entryId, { status: 'error', error: 'cancelled' });
				return;
			}
			const existing = new Set(this.matches.map((m) => m.infoHash));
			const additions = fresh.filter((t) => !existing.has(t.infoHash));
			if (additions.length > 0) this.matches = [...this.matches, ...additions];
			this.updateStackEntry(entryId, { status: 'done', count: additions.length });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			console.warn('[torrent-search] append failed:', err);
			this.updateStackEntry(entryId, { status: 'error', error: message });
		}
	}

	async search(args: RunArgs): Promise<void> {
		const myRun = ++this.run;
		this.status = 'searching';
		this.error = null;
		this.matches = [];
		this.rowEvals = {};
		this.searchStack = [];
		const entryId = this.pushStackEntry('Show', args.title);
		try {
			const kind = addonKind(args.addon);
			const isTv = kind === 'tv show';
			const isMovie = kind === 'movie';
			let raw: RawTorrent[] | null = null;
			let evals: Record<string, unknown> = {};

			if (args.firkinId) {
				if (!args.forceRefresh) {
					const cached = await fetchFirkinTorrentSearch(args.firkinId);
					if (myRun !== this.run) return;
					if (cached && Array.isArray(cached.results) && cached.results.length > 0) {
						raw = cached.results;
						evals = cached.evals ?? {};
					}
				}
				if (raw === null) {
					const category = tpbCategoryFor(args.addon) ?? '0';
					const refreshed = await refreshFirkinTorrentSearch(
						args.firkinId,
						args.addon,
						args.title,
						String(category)
					);
					if (myRun !== this.run) return;
					if (refreshed && Array.isArray(refreshed.results)) {
						raw = refreshed.results;
						evals = refreshed.evals ?? {};
					}
				}
			}

			const torrents = raw
				? raw.map((t) => ({ ...t, ...parseTorrentName(t.title) }))
				: await searchTorrents(args.addon, args.title);
			if (myRun !== this.run) return;
			const matches = matchTorrentsForResult(
				{
					title: args.title,
					description: '',
					artists: [],
					images: [],
					files: [],
					year: args.year,
					raw: null
				},
				torrents,
				{ skipYearFilter: isTv, excludeTvSeries: isMovie }
			);
			this.matches = matches;

			const seedEvals: Record<string, TorrentRowEval> = {};
			for (const m of matches) {
				if (!m.magnetLink) continue;
				const cached = evalToRowEval(evals[m.infoHash]);
				if (cached) seedEvals[m.magnetLink] = cached;
			}
			this.rowEvals = seedEvals;

			this.status = 'done';
			this.updateStackEntry(entryId, { status: 'done', count: matches.length });
			// Streamability probing runs one /api/torrent/evaluate per row.
			// For TV shows the result set spans every season the show ever
			// aired, so the per-row probe is way too expensive (and the
			// streamability column lives in CatalogTorrentSearchCard, which
			// TV firkins skip in favour of the seasons-card layout).
			if (this.evaluate && !isTv) await this.evaluateGrouped(groupMatches(matches), myRun);
		} catch (err) {
			if (myRun !== this.run) return;
			this.matches = [];
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.error = message;
			this.status = 'error';
			this.updateStackEntry(entryId, { status: 'error', error: message });
		}
	}

	// Each defined quality group kicks off its own sequential probe queue at
	// the same time, so groups run in parallel and within a group rows go
	// top-down in peer-score order (already sorted by groupMatches), stopping
	// the moment one is streamable. Remaining rows in the group are marked
	// 'skipped' so the UI dims them. Groups with `probe: false` (the "Other"
	// catch-all for unknown quality strings) are not probed at all — their
	// rows are pre-marked 'skipped' too so the UI doesn't leave them spinning.
	private async evaluateGrouped(grouped: TorrentQualityGroup[], runToken: number): Promise<void> {
		const seed: Record<string, TorrentRowEval> = {};
		for (const g of grouped) {
			for (const t of g.rows) {
				if (!t.magnetLink) continue;
				// Pre-populated cache entries (loaded by `search()` from
				// `/api/firkins/:id/torrent-search`) win over fresh seeding —
				// don't overwrite them with `pending` / `skipped`.
				if (this.rowEvals[t.magnetLink]) continue;
				if (t.seeders <= 0) {
					seed[t.magnetLink] = { kind: 'skipped', reason: 'No seeders' };
					continue;
				}
				seed[t.magnetLink] = g.probe
					? { kind: 'pending' }
					: { kind: 'skipped', reason: 'Unknown quality — not probed' };
			}
		}
		this.rowEvals = { ...this.rowEvals, ...seed };

		await Promise.all(grouped.filter((g) => g.probe).map((g) => this.probeGroup(g, runToken)));
	}

	/// Probe rows in a group that were initially skipped because an earlier
	/// row in the same quality bucket already proved streamable. Used when
	/// the user expands the row via "More" — they want to compare options,
	/// so re-evaluate the rest. Leaves 'no seeders' / 'unknown quality' /
	/// already-evaluated rows alone.
	async probeRemaining(groupLabel: string): Promise<void> {
		const runToken = this.run;
		const group = this.groupedMatches.find((g) => g.label === groupLabel);
		if (!group || !group.probe) return;
		for (const t of group.rows) {
			if (runToken !== this.run) return;
			if (!t.magnetLink) continue;
			const current = this.rowEvals[t.magnetLink];
			if (current?.kind !== 'skipped') continue;
			if (current.reason !== 'Streamable candidate found earlier in this quality group') continue;
			this.rowEvals = { ...this.rowEvals, [t.magnetLink]: { kind: 'evaluating' } };
			const result = await this.probeOne(t.magnetLink);
			if (runToken !== this.run) return;
			this.rowEvals = { ...this.rowEvals, [t.magnetLink]: result };
		}
	}

	private async probeGroup(group: TorrentQualityGroup, runToken: number): Promise<void> {
		let foundStreamable = false;
		for (const t of group.rows) {
			if (runToken !== this.run) return;
			if (!t.magnetLink) continue;
			const current = this.rowEvals[t.magnetLink];
			// Cached results (loaded from the firkin's persisted search) take
			// precedence — don't re-probe rows we already know about.
			if (current?.kind === 'streamable') {
				foundStreamable = true;
				continue;
			}
			if (current?.kind === 'not-streamable') continue;
			if (current?.kind === 'skipped') continue;
			if (t.seeders <= 0) continue;
			if (foundStreamable) {
				this.rowEvals = {
					...this.rowEvals,
					[t.magnetLink]: {
						kind: 'skipped',
						reason: 'Streamable candidate found earlier in this quality group'
					}
				};
				continue;
			}
			this.rowEvals = { ...this.rowEvals, [t.magnetLink]: { kind: 'evaluating' } };
			const result = await this.probeOne(t.magnetLink);
			if (runToken !== this.run) return;
			this.rowEvals = { ...this.rowEvals, [t.magnetLink]: result };
			if (result.kind === 'streamable') foundStreamable = true;
		}
	}

	private async probeOne(magnet: string): Promise<TorrentRowEval> {
		try {
			const res = await fetch(`${base}/api/torrent/evaluate`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ magnet })
			});
			const body = (await res.json()) as
				| {
						streamable: true;
						fileName: string;
						fileSize: number;
						mimeType: string | null;
				  }
				| { streamable: false; reason: string };
			if (body.streamable) {
				return {
					kind: 'streamable',
					fileName: body.fileName,
					fileSize: body.fileSize,
					mimeType: body.mimeType
				};
			}
			return { kind: 'not-streamable', reason: body.reason };
		} catch (err) {
			const reason = err instanceof Error ? err.message : 'Unknown error';
			return { kind: 'not-streamable', reason };
		}
	}
}

export async function startTorrentDownload(magnet: string): Promise<void> {
	const res = await fetch(`${base}/api/torrent/add`, {
		method: 'POST',
		headers: { 'content-type': 'application/json' },
		body: JSON.stringify({ magnet })
	});
	if (!res.ok) {
		let message = `HTTP ${res.status}`;
		try {
			const body = await res.json();
			if (body && typeof body.error === 'string') message = body.error;
		} catch {
			// ignore
		}
		throw new Error(message);
	}
}
