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
	| { kind: 'not-streamable'; reason: string };

export interface TorrentSearchOptions {
	/** When true, runs `/api/torrent/evaluate` for each result so the UI can mark streamability. */
	evaluate?: boolean;
	concurrency?: number;
}

interface RunArgs {
	addon: string;
	title: string;
	year: number | null;
}

const DEFAULT_EVAL_CONCURRENCY = 4;

export class TorrentSearch {
	matches = $state<TorrentResultItem[]>([]);
	status = $state<TorrentSearchStatus>('idle');
	error = $state<string | null>(null);
	rowEvals = $state<Record<string, TorrentRowEval>>({});

	private run = 0;
	private readonly evaluate: boolean;
	private readonly concurrency: number;

	constructor(options: TorrentSearchOptions = {}) {
		this.evaluate = options.evaluate ?? false;
		this.concurrency = options.concurrency ?? DEFAULT_EVAL_CONCURRENCY;
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
			if (this.evaluate) await this.evaluateAll(matches, myRun);
		} catch (err) {
			if (myRun !== this.run) return;
			this.matches = [];
			this.error = err instanceof Error ? err.message : 'Unknown error';
			this.status = 'error';
		}
	}

	private async evaluateAll(matches: TorrentResultItem[], runToken: number): Promise<void> {
		const seed: Record<string, TorrentRowEval> = {};
		for (const t of matches) {
			if (t.magnetLink) seed[t.magnetLink] = { kind: 'pending' };
		}
		this.rowEvals = seed;

		let cursor = 0;
		const next = (): TorrentResultItem | null => {
			while (cursor < matches.length) {
				const t = matches[cursor++];
				if (t.magnetLink) return t;
			}
			return null;
		};

		const worker = async () => {
			while (runToken === this.run) {
				const t = next();
				if (!t || !t.magnetLink) break;
				this.rowEvals = { ...this.rowEvals, [t.magnetLink]: { kind: 'evaluating' } };
				let result: TorrentRowEval;
				try {
					const res = await fetch(`${base}/api/torrent/evaluate`, {
						method: 'POST',
						headers: { 'content-type': 'application/json' },
						body: JSON.stringify({ magnet: t.magnetLink })
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
						result = {
							kind: 'streamable',
							fileName: body.fileName,
							fileSize: body.fileSize,
							mimeType: body.mimeType
						};
					} else {
						result = { kind: 'not-streamable', reason: body.reason };
					}
				} catch (err) {
					const reason = err instanceof Error ? err.message : 'Unknown error';
					result = { kind: 'not-streamable', reason };
				}
				if (runToken !== this.run) return;
				this.rowEvals = { ...this.rowEvals, [t.magnetLink]: result };
			}
		};

		const pool = Math.min(this.concurrency, matches.length);
		await Promise.all(Array.from({ length: pool }, () => worker()));
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
