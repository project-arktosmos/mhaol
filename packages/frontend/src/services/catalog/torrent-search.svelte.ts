import { base } from '$app/paths';
import {
	matchTorrentsForResult,
	searchTorrents,
	type TorrentResultItem
} from '$lib/search.service';
import { addonKind } from '$lib/firkins.service';

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
		list.sort((a, b) => b.seeders + b.leechers - (a.seeders + a.leechers));
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
			const isTv = addonKind(addon) === 'tv show';
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
				{ skipYearFilter: isTv }
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
			const torrents = await searchTorrents(args.addon, args.title);
			if (myRun !== this.run) return;
			const isTv = addonKind(args.addon) === 'tv show';
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
				{ skipYearFilter: isTv }
			);
			this.matches = matches;
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
				if (t.seeders <= 0) {
					seed[t.magnetLink] = { kind: 'skipped', reason: 'No seeders' };
					continue;
				}
				seed[t.magnetLink] = g.probe
					? { kind: 'pending' }
					: { kind: 'skipped', reason: 'Unknown quality — not probed' };
			}
		}
		this.rowEvals = seed;

		await Promise.all(grouped.filter((g) => g.probe).map((g) => this.probeGroup(g, runToken)));
	}

	private async probeGroup(group: TorrentQualityGroup, runToken: number): Promise<void> {
		let foundStreamable = false;
		for (const t of group.rows) {
			if (runToken !== this.run) return;
			if (!t.magnetLink) continue;
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
