import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from 'frontend/lib/api-base';
import type { UnifiedDownload } from 'frontend/types/download.type';

export interface DownloadsState {
	downloads: UnifiedDownload[];
	loading: boolean;
	error: string | null;
}

const initialState: DownloadsState = {
	downloads: [],
	loading: false,
	error: null
};

const POLL_INTERVAL_MS = 3000;

class DownloadsService {
	public state: Writable<DownloadsState> = writable(initialState);
	public modalOpen: Writable<boolean> = writable(false);

	private pollTimer: ReturnType<typeof setInterval> | null = null;
	private subscribers = 0;

	openModal(): void {
		this.modalOpen.set(true);
	}

	closeModal(): void {
		this.modalOpen.set(false);
	}

	startPolling(): void {
		if (!browser) return;
		this.subscribers++;
		if (this.pollTimer) return;

		this.fetchDownloads(true);
		this.pollTimer = setInterval(() => this.fetchDownloads(false), POLL_INTERVAL_MS);
	}

	stopPolling(): void {
		if (!browser) return;
		this.subscribers = Math.max(0, this.subscribers - 1);
		if (this.subscribers > 0 || !this.pollTimer) return;

		clearInterval(this.pollTimer);
		this.pollTimer = null;
	}

	private async fetchDownloads(showLoading: boolean): Promise<void> {
		if (showLoading) {
			this.state.update((s) => ({ ...s, loading: true, error: null }));
		}

		try {
			const res = await fetch(apiUrl('/api/downloads'));
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const downloads: UnifiedDownload[] = await res.json();
			this.state.update((s) => ({ ...s, downloads, loading: false, error: null }));
		} catch (e) {
			if (showLoading) {
				const msg = e instanceof Error ? e.message : String(e);
				this.state.update((s) => ({ ...s, loading: false, error: msg }));
			}
		}
	}
}

export const downloadsService = new DownloadsService();
