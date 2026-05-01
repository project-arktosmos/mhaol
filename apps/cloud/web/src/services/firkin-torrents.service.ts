import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchJson } from '$transport/fetch-helpers';
import type { TorrentInfo } from '$types/torrent.type';

export interface FirkinTorrentsState {
	byHash: Record<string, TorrentInfo>;
	loading: boolean;
	error: string | null;
}

const initialState: FirkinTorrentsState = {
	byHash: {},
	loading: false,
	error: null
};

const POLL_INTERVAL_MS = 2000;

export function infoHashFromMagnet(magnet: string): string | null {
	if (!magnet.startsWith('magnet:')) return null;
	const idx = magnet.indexOf('btih:');
	if (idx === -1) return null;
	const tail = magnet.slice(idx + 'btih:'.length);
	const end = tail.search(/[&]/);
	const raw = (end === -1 ? tail : tail.slice(0, end)).trim();
	return raw ? raw.toLowerCase() : null;
}

class FirkinTorrentsService {
	state: Writable<FirkinTorrentsState> = writable(initialState);

	private subscribers = 0;
	private timer: ReturnType<typeof setInterval> | null = null;
	private inFlight = false;

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
		try {
			const list = await fetchJson<TorrentInfo[]>('/api/torrent/list');
			const byHash: Record<string, TorrentInfo> = {};
			for (const t of list) {
				byHash[t.infoHash.toLowerCase()] = t;
			}
			this.state.set({ byHash, loading: false, error: null });
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, loading: false, error: message }));
		} finally {
			this.inFlight = false;
		}
	}

	async add(magnet: string): Promise<TorrentInfo | null> {
		if (!browser) return null;
		try {
			const info = await fetchJson<TorrentInfo>('/api/torrent/add', {
				method: 'POST',
				body: JSON.stringify({ magnet })
			});
			this.state.update((s) => ({
				...s,
				byHash: { ...s.byHash, [info.infoHash.toLowerCase()]: info }
			}));
			return info;
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({ ...s, error: message }));
			return null;
		}
	}
}

export const firkinTorrentsService = new FirkinTorrentsService();
