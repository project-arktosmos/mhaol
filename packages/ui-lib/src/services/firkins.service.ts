import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchJson } from 'ui-lib/transport/fetch-helpers';
import type { CloudFirkin, FirkinArtist, FirkinFile, FirkinImage } from 'ui-lib/types/firkin.type';

export interface CreateFirkinInput {
	title: string;
	artists: FirkinArtist[];
	description: string;
	images: FirkinImage[];
	files: FirkinFile[];
	year: number | null;
	type: string;
	source: string;
}

export interface FirkinsServiceState {
	loading: boolean;
	firkins: CloudFirkin[];
	error: string | null;
}

const initialState: FirkinsServiceState = {
	loading: false,
	firkins: [],
	error: null
};

const POLL_INTERVAL_MS = 4000;

class FirkinsService {
	public state: Writable<FirkinsServiceState> = writable(initialState);

	private subscribers = 0;
	private timer: ReturnType<typeof setInterval> | null = null;
	private inFlight = false;

	/** Begin refcounted polling. Returns a cleanup that stops polling once the last consumer leaves. */
	start(): () => void {
		this.subscribers += 1;
		if (this.subscribers === 1 && browser) {
			void this.refresh();
			this.timer = setInterval(() => {
				void this.refresh();
			}, POLL_INTERVAL_MS);
		}
		return () => this.stop();
	}

	private stop(): void {
		this.subscribers = Math.max(0, this.subscribers - 1);
		if (this.subscribers === 0 && this.timer) {
			clearInterval(this.timer);
			this.timer = null;
		}
	}

	async refresh(): Promise<void> {
		if (!browser || this.inFlight) return;
		this.inFlight = true;
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const firkins = await fetchJson<CloudFirkin[]>('/api/firkins');
			this.state.set({ loading: false, firkins, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		} finally {
			this.inFlight = false;
		}
	}

	async create(input: CreateFirkinInput): Promise<CloudFirkin> {
		const created = await fetchJson<CloudFirkin>('/api/firkins', {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify(input)
		});
		this.state.update((s) => {
			const idx = s.firkins.findIndex((d) => d.id === created.id);
			if (idx >= 0) {
				const next = s.firkins.slice();
				next[idx] = created;
				return { ...s, firkins: next };
			}
			return { ...s, firkins: [...s.firkins, created] };
		});
		return created;
	}
}

export const firkinsService = new FirkinsService();
