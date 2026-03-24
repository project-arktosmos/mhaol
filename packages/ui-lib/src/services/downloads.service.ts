import { writable, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchRaw } from 'ui-lib/transport/fetch-helpers';
import type { UnifiedDownload } from 'ui-lib/types/download.type';

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

	async pauseDownload(dl: UnifiedDownload): Promise<void> {
		if (dl.type === 'torrent') {
			await fetchRaw(`/api/torrent/torrents/${encodeURIComponent(dl.id)}/pause`, {
				method: 'POST'
			});
		}
		await this.fetchDownloads(false);
	}

	async resumeDownload(dl: UnifiedDownload): Promise<void> {
		if (dl.type === 'torrent') {
			await fetchRaw(`/api/torrent/torrents/${encodeURIComponent(dl.id)}/resume`, {
				method: 'POST'
			});
		}
		await this.fetchDownloads(false);
	}

	async removeDownload(dl: UnifiedDownload, deleteFiles = false): Promise<void> {
		if (dl.type === 'torrent') {
			const qs = deleteFiles ? '?delete_files=true' : '';
			await fetchRaw(`/api/torrent/torrents/${encodeURIComponent(dl.id)}${qs}`, {
				method: 'DELETE'
			});
		} else {
			await fetchRaw(`/api/ytdl/downloads/${encodeURIComponent(dl.id)}`, {
				method: 'DELETE'
			});
		}
		await this.fetchDownloads(false);
	}

	private async fetchDownloads(showLoading: boolean): Promise<void> {
		if (showLoading) {
			this.state.update((s) => ({ ...s, loading: true, error: null }));
		}

		try {
			const res = await fetchRaw('/api/downloads');
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
