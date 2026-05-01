import { writable, type Writable } from 'svelte/store';

export type TorrentState =
	| 'initializing'
	| 'downloading'
	| 'seeding'
	| 'paused'
	| 'checking'
	| 'error';

export interface TorrentInfo {
	id?: string;
	infoHash: string;
	name: string;
	size: number;
	progress: number;
	downloadSpeed: number;
	uploadSpeed: number;
	peers: number;
	seeds: number;
	state: TorrentState;
	addedAt: number;
	eta: number | null;
	outputPath: string | null;
}

export interface TorrentStatusState {
	loading: boolean;
	torrents: TorrentInfo[];
	error: string | null;
	lastCheckedAt: number | null;
}

const initialState: TorrentStatusState = {
	loading: false,
	torrents: [],
	error: null,
	lastCheckedAt: null
};

async function parseError(res: Response): Promise<string> {
	try {
		const data = await res.json();
		if (data && typeof data.error === 'string') return data.error;
	} catch {
		// fall through
	}
	return `HTTP ${res.status}`;
}

class TorrentStatusService {
	state: Writable<TorrentStatusState> = writable(initialState);
	private timer: ReturnType<typeof setInterval> | null = null;

	async refresh(): Promise<void> {
		this.state.update((s) => ({ ...s, loading: true, error: null }));
		try {
			const res = await fetch('/api/torrent/list', { cache: 'no-store' });
			if (!res.ok) throw new Error(await parseError(res));
			const torrents = (await res.json()) as TorrentInfo[];
			this.state.set({
				loading: false,
				torrents,
				error: null,
				lastCheckedAt: Date.now()
			});
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Unknown error';
			this.state.update((s) => ({
				...s,
				loading: false,
				error: message,
				lastCheckedAt: Date.now()
			}));
		}
	}

	start(intervalMs: number = 5000): void {
		this.refresh();
		this.stop();
		this.timer = setInterval(() => this.refresh(), intervalMs);
	}

	stop(): void {
		if (this.timer !== null) {
			clearInterval(this.timer);
			this.timer = null;
		}
	}
}

export const torrentStatusService = new TorrentStatusService();
