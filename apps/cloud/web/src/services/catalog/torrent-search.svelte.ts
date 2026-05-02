import { base } from '$app/paths';
import {
	matchTorrentsForResult,
	searchTorrents,
	type TorrentResultItem
} from '$lib/search.service';

export type TorrentSearchStatus = 'idle' | 'searching' | 'done' | 'error';

export type TorrentRowEval =
	| { kind: 'pending' }
	| { kind: 'evaluating' }
	| { kind: 'streamable'; fileName: string; fileSize: number; mimeType: string | null }
	| { kind: 'not-streamable'; reason: string }
	| { kind: 'skipped' };

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
	rows: TorrentResultItem[];
}

const QUALITY_GROUPS: { label: string; matches: (q: string | null) => boolean }[] = [
	{ label: '4K / 2160p', matches: (q) => q === '2160p' || q === '4K' || q === '4K UHD' },
	{ label: '1080p', matches: (q) => q === '1080p' },
	{ label: '720p', matches: (q) => q === '720p' },
	{ label: '480p', matches: (q) => q === '480p' },
	{ label: '360p', matches: (q) => q === '360p' },
	{ label: 'Other', matches: () => true }
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
		if (rows && rows.length > 0) out.push({ label: QUALITY_GROUPS[i].label, rows });
	}
	return out;
}

export class TorrentSearch {
	matches = $state<TorrentResultItem[]>([]);
	status = $state<TorrentSearchStatus>('idle');
	error = $state<string | null>(null);
	rowEvals = $state<Record<string, TorrentRowEval>>({});
	groupedMatches = $derived<TorrentQualityGroup[]>(groupMatches(this.matches));

	private run = 0;
	private readonly evaluate: boolean;

	constructor(options: TorrentSearchOptions = {}) {
		this.evaluate = options.evaluate ?? false;
	}

	cancel(): void {
		this.run++;
	}

	async search(args: RunArgs): Promise<void> {
		const myRun = ++this.run;
		this.status = 'searching';
		this.error = null;
		this.matches = [];
		this.rowEvals = {};
		try {
			const torrents = await searchTorrents(args.addon, args.title);
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
				torrents
			);
			this.matches = matches;
			this.status = 'done';
			if (this.evaluate) await this.evaluateGrouped(groupMatches(matches), myRun);
		} catch (err) {
			if (myRun !== this.run) return;
			this.matches = [];
			this.error = err instanceof Error ? err.message : 'Unknown error';
			this.status = 'error';
		}
	}

	// Probe each quality group sequentially. Within a group, walk rows in
	// peer-score order (already sorted by groupMatches) and stop the moment
	// one is streamable; remaining rows in that group are marked 'skipped'
	// so the UI can dim them. Then move on to the next group.
	private async evaluateGrouped(grouped: TorrentQualityGroup[], runToken: number): Promise<void> {
		const seed: Record<string, TorrentRowEval> = {};
		for (const g of grouped) {
			for (const t of g.rows) {
				if (t.magnetLink) seed[t.magnetLink] = { kind: 'pending' };
			}
		}
		this.rowEvals = seed;

		for (const g of grouped) {
			if (runToken !== this.run) return;
			let foundStreamable = false;
			for (const t of g.rows) {
				if (runToken !== this.run) return;
				if (!t.magnetLink) continue;
				if (foundStreamable) {
					this.rowEvals = { ...this.rowEvals, [t.magnetLink]: { kind: 'skipped' } };
					continue;
				}
				this.rowEvals = { ...this.rowEvals, [t.magnetLink]: { kind: 'evaluating' } };
				const result = await this.probeOne(t.magnetLink);
				if (runToken !== this.run) return;
				this.rowEvals = { ...this.rowEvals, [t.magnetLink]: result };
				if (result.kind === 'streamable') foundStreamable = true;
			}
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
