import { base } from '$app/paths';
import type { SubsLyricsItem } from '$types/subs-lyrics.type';

export type SubsLyricsResolverStatus = 'idle' | 'searching' | 'done' | 'error';

interface SearchArgs {
	addon: string;
	query: string;
	externalIds?: string[];
	languages?: string[];
}

export class SubsLyricsResolver {
	results = $state<SubsLyricsItem[]>([]);
	status = $state<SubsLyricsResolverStatus>('idle');
	error = $state<string | null>(null);

	private run = 0;

	cancel(): void {
		this.run++;
	}

	async search(args: SearchArgs): Promise<void> {
		const myRun = ++this.run;
		this.status = 'searching';
		this.error = null;
		this.results = [];
		try {
			const res = await fetch(`${base}/api/search/subs-lyrics`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({
					addon: args.addon,
					query: args.query,
					externalIds: args.externalIds ?? [],
					languages: args.languages
				})
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
			const body = (await res.json()) as SubsLyricsItem[];
			if (myRun !== this.run) return;
			this.results = Array.isArray(body) ? body : [];
			this.status = 'done';
		} catch (err) {
			if (myRun !== this.run) return;
			this.error = err instanceof Error ? err.message : 'Unknown error';
			this.status = 'error';
		}
	}
}
