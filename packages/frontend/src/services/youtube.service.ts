import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { apiUrl } from '$lib/api-base';
import {
	extractVideoId,
	type YouTubeSettings,
	type YouTubeServiceState,
	type YouTubeDownloadProgress,
	type YouTubeVideoInfo,
	type YouTubePlaylistInfo,
	type YouTubeManagerStats,
	type YouTubeConfig,
	type DownloaderStatus,
	type AudioQuality,
	type AudioFormat,
	type DownloadMode,
	type VideoQuality,
	type VideoFormat
} from '$types/youtube.type';

const API_PREFIX = '/api/ytdl';

// Default settings (used before server fetch completes)
const initialSettings: YouTubeSettings = {
	id: 'youtube-settings',
	downloadMode: 'audio',
	defaultQuality: 'high',
	defaultFormat: 'aac',
	defaultVideoQuality: 'best',
	defaultVideoFormat: 'mp4',
	libraryId: '',
	poToken: '',
	cookies: ''
};

// Initial service state
const initialState: YouTubeServiceState = {
	initialized: false,
	loading: false,
	error: null,
	libraryId: '',
	downloads: [],
	stats: null,
	downloaderStatus: null,
	currentUrl: '',
	currentVideoInfo: null,
	currentPlaylistInfo: null,
	fetchingInfo: false,
	fetchingVideoInfo: false,
	fetchingPlaylistInfo: false
};

class YouTubeService {
	public store: Writable<YouTubeSettings> = writable(initialSettings);
	public state: Writable<YouTubeServiceState> = writable(initialState);

	private eventSource: EventSource | null = null;
	private _initialized = false;

	// ===== Settings Access =====

	get(): YouTubeSettings {
		return get(this.store);
	}

	// ===== Initialization =====

	async initialize(): Promise<void> {
		if (!browser || this._initialized) return;

		// Clean up legacy localStorage entry
		localStorage.removeItem('object-service:youtube-settings');

		this.state.update((s) => ({ ...s, loading: true }));

		try {
			const [stats, downloaderStatus, settings] = await Promise.all([
				this.fetchJson<YouTubeManagerStats>('/api/ytdl/status'),
				this.fetchJson<DownloaderStatus>('/api/ytdl/ytdlp/status'),
				this.fetchJson<Omit<YouTubeSettings, 'id'>>('/api/ytdl/settings')
			]);

			// Populate the settings store from database
			this.store.set({ ...settings, id: 'youtube-settings' });

			this.state.update((s) => ({
				...s,
				initialized: true,
				loading: false,
				libraryId: settings.libraryId,
				stats,
				downloaderStatus,
				error: null
			}));

			this._initialized = true;

			// Connect SSE for real-time updates
			this.connectSSE();
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				loading: false,
				error: `Failed to connect to download server: ${errorMsg}`
			}));
		}
	}

	// ===== Video Info =====

	async fetchVideoInfo(url: string): Promise<YouTubeVideoInfo | null> {
		if (!browser) return null;

		this.state.update((s) => ({
			...s,
			currentUrl: url,
			fetchingInfo: true,
			fetchingVideoInfo: true,
			currentVideoInfo: null,
			currentPlaylistInfo: null,
			error: null
		}));

		try {
			const info = await this.fetchJson<YouTubeVideoInfo>(
				`/api/ytdl/info/video?url=${encodeURIComponent(url)}`
			);

			this.state.update((s) => ({
				...s,
				currentVideoInfo: info,
				fetchingInfo: false,
				fetchingVideoInfo: false
			}));

			return info;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				fetchingInfo: false,
				fetchingVideoInfo: false,
				error: `Failed to fetch video info: ${errorMsg}`
			}));
			return null;
		}
	}

	// ===== Playlist Info =====

	async fetchPlaylistInfo(url: string): Promise<YouTubePlaylistInfo | null> {
		if (!browser) return null;

		this.state.update((s) => ({
			...s,
			currentUrl: url,
			fetchingInfo: true,
			fetchingPlaylistInfo: true,
			currentVideoInfo: null,
			currentPlaylistInfo: null,
			error: null
		}));

		try {
			const info = await this.fetchJson<YouTubePlaylistInfo>(
				`/api/ytdl/info/playlist?url=${encodeURIComponent(url)}`
			);

			this.state.update((s) => ({
				...s,
				currentPlaylistInfo: info,
				fetchingInfo: false,
				fetchingPlaylistInfo: false
			}));

			return info;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				fetchingInfo: false,
				fetchingPlaylistInfo: false,
				error: `Failed to fetch playlist info: ${errorMsg}`
			}));
			return null;
		}
	}

	setCurrentUrl(url: string): void {
		this.state.update((s) => ({
			...s,
			currentUrl: url,
			currentVideoInfo: null,
			currentPlaylistInfo: null,
			error: null
		}));
	}

	clearCurrentVideo(): void {
		this.state.update((s) => ({
			...s,
			currentUrl: '',
			currentVideoInfo: null,
			currentPlaylistInfo: null,
			error: null
		}));
	}

	// ===== Downloads =====

	async downloadAudio(): Promise<string | null> {
		return this.download();
	}

	async download(): Promise<string | null> {
		if (!browser) return null;

		const currentState = get(this.state);
		const settings = this.get();

		if (!currentState.currentUrl) {
			this.state.update((s) => ({ ...s, error: 'No URL provided' }));
			return null;
		}

		const videoInfo = currentState.currentVideoInfo;

		try {
			const body: Record<string, unknown> = {
				url: currentState.currentUrl,
				videoId: videoInfo?.videoId || extractVideoId(currentState.currentUrl) || '',
				title: videoInfo?.title || 'Unknown',
				mode: settings.downloadMode,
				quality: settings.defaultQuality,
				format: settings.defaultFormat
			};

			if (settings.downloadMode === 'video') {
				body.videoQuality = settings.defaultVideoQuality;
				body.videoFormat = settings.defaultVideoFormat;
			}

			const result = await this.fetchJson<{ downloadId: string }>('/api/ytdl/downloads', {
				method: 'POST',
				body: JSON.stringify(body)
			});

			this.clearCurrentVideo();
			return result.downloadId;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to start download: ${errorMsg}`
			}));
			return null;
		}
	}

	async cancelDownload(downloadId: string): Promise<void> {
		if (!browser) return;

		try {
			await this.fetchJson(`/api/ytdl/downloads/${downloadId}`, { method: 'DELETE' });
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to cancel download: ${errorMsg}`
			}));
		}
	}

	async clearCompleted(): Promise<void> {
		if (!browser) return;

		try {
			await this.fetchJson('/api/ytdl/downloads/completed', { method: 'DELETE' });
			// Refresh downloads list since SSE might not trigger for removed items
			const downloads = await this.fetchJson<YouTubeDownloadProgress[]>('/api/ytdl/downloads');
			this.state.update((s) => ({ ...s, downloads }));
		} catch (error) {
			console.error('[YouTube] Failed to clear completed:', error);
		}
	}

	// ===== Playlist Downloads =====

	async downloadPlaylist(): Promise<string[] | null> {
		if (!browser) return null;

		const state = get(this.state);
		const settings = this.get();

		if (!state.currentPlaylistInfo) {
			this.state.update((s) => ({ ...s, error: 'No playlist loaded' }));
			return null;
		}

		const videos = state.currentPlaylistInfo.videos.map((v) => ({
			url: `https://www.youtube.com/watch?v=${v.videoId}`,
			videoId: v.videoId,
			title: v.title
		}));

		const body: Record<string, unknown> = {
			videos,
			mode: settings.downloadMode,
			quality: settings.defaultQuality,
			format: settings.defaultFormat
		};

		if (settings.downloadMode === 'video') {
			body.videoQuality = settings.defaultVideoQuality;
			body.videoFormat = settings.defaultVideoFormat;
		}

		try {
			const result = await this.fetchJson<{ downloadIds: string[] }>(
				'/api/ytdl/downloads/playlist',
				{
					method: 'POST',
					body: JSON.stringify(body)
				}
			);

			this.clearCurrentVideo();
			return result.downloadIds;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to queue playlist: ${errorMsg}`
			}));
			return null;
		}
	}

	async queueSingleDownload(url: string, videoId: string, title: string): Promise<string | null> {
		if (!browser) return null;

		const settings = this.get();

		const body: Record<string, unknown> = {
			url,
			videoId,
			title,
			mode: settings.downloadMode,
			quality: settings.defaultQuality,
			format: settings.defaultFormat
		};

		if (settings.downloadMode === 'video') {
			body.videoQuality = settings.defaultVideoQuality;
			body.videoFormat = settings.defaultVideoFormat;
		}

		try {
			const result = await this.fetchJson<{ downloadId: string }>('/api/ytdl/downloads', {
				method: 'POST',
				body: JSON.stringify(body)
			});

			return result.downloadId;
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to queue download: ${errorMsg}`
			}));
			return null;
		}
	}

	async clearQueue(): Promise<void> {
		if (!browser) return;

		try {
			await this.fetchJson('/api/ytdl/downloads/queue', { method: 'DELETE' });
		} catch (error) {
			console.error('[YouTube] Failed to clear queue:', error);
		}
	}

	// ===== Settings Management (database-backed) =====

	async updateSettings(updates: Partial<YouTubeSettings>): Promise<void> {
		if (!browser) return;

		const current = this.get();
		const merged = { ...current, ...updates };

		// Optimistic update
		this.store.set(merged);

		// Strip 'id' before sending to server
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
		const { id, ...payload } = updates as Partial<YouTubeSettings> & { id?: unknown };

		try {
			await this.fetchJson('/api/ytdl/settings', {
				method: 'PUT',
				body: JSON.stringify(payload)
			});

			if (updates.libraryId !== undefined) {
				this.state.update((s) => ({ ...s, libraryId: updates.libraryId! }));
			}
		} catch (error) {
			// Revert on failure
			this.store.set(current);
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to save settings: ${errorMsg}`
			}));
		}
	}

	setDownloadMode(mode: DownloadMode): void {
		this.updateSettings({ downloadMode: mode });
	}

	setDefaultQuality(quality: AudioQuality): void {
		this.updateSettings({ defaultQuality: quality });
	}

	setDefaultFormat(format: AudioFormat): void {
		this.updateSettings({ defaultFormat: format });
	}

	setDefaultVideoQuality(quality: VideoQuality): void {
		this.updateSettings({ defaultVideoQuality: quality });
	}

	setDefaultVideoFormat(format: VideoFormat): void {
		this.updateSettings({ defaultVideoFormat: format });
	}

	setLibrary(libraryId: string): void {
		this.updateSettings({ libraryId });
	}

	// ===== Getters =====

	get isInitialized(): boolean {
		return get(this.state).initialized;
	}

	get hasActiveDownloads(): boolean {
		const stats = get(this.state).stats;
		return stats ? stats.activeDownloads > 0 : false;
	}

	get hasPendingWork(): boolean {
		const stats = get(this.state).stats;
		return stats ? stats.activeDownloads > 0 || stats.queuedDownloads > 0 : false;
	}

	// ===== Authentication Config =====

	async setConfig(config: YouTubeConfig): Promise<void> {
		if (!browser) return;

		try {
			await this.fetchJson('/api/ytdl/settings', {
				method: 'PUT',
				body: JSON.stringify({
					poToken: config.poToken ?? '',
					cookies: config.cookies ?? ''
				})
			});
		} catch (error) {
			const errorMsg = error instanceof Error ? error.message : String(error);
			this.state.update((s) => ({
				...s,
				error: `Failed to set config: ${errorMsg}`
			}));
		}
	}

	getAuthConfig(): YouTubeConfig {
		const settings = this.get();
		return {
			poToken: settings.poToken || null,
			cookies: settings.cookies || null
		};
	}

	// ===== Downloader Status =====

	async refreshDownloaderStatus(): Promise<void> {
		if (!browser) return;

		try {
			const status = await this.fetchJson<DownloaderStatus>('/api/ytdl/ytdlp/status');
			this.state.update((s) => ({ ...s, downloaderStatus: status }));
		} catch {
			// ignore
		}
	}

	// ===== SSE Connection =====

	private connectSSE(): void {
		if (!browser) return;

		this.eventSource = new EventSource(apiUrl(`${API_PREFIX}/downloads/events`));

		this.eventSource.addEventListener('progress', (e: MessageEvent) => {
			try {
				const progress = JSON.parse(e.data) as YouTubeDownloadProgress;
				this.state.update((s) => {
					const idx = s.downloads.findIndex((d) => d.downloadId === progress.downloadId);
					const downloads = [...s.downloads];
					if (idx >= 0) {
						downloads[idx] = progress;
					} else {
						downloads.push(progress);
					}
					return { ...s, downloads };
				});
			} catch {
				// ignore parse errors
			}
		});

		this.eventSource.addEventListener('stats', (e: MessageEvent) => {
			try {
				const stats = JSON.parse(e.data) as YouTubeManagerStats;
				this.state.update((s) => ({ ...s, stats }));
			} catch {
				// ignore parse errors
			}
		});

		this.eventSource.onerror = () => {
			// EventSource auto-reconnects, but we could update state
			console.warn('[YouTube] SSE connection error, reconnecting...');
		};
	}

	// ===== HTTP Helper =====

	private async fetchJson<T>(path: string, init?: RequestInit): Promise<T> {
		const response = await fetch(apiUrl(path), {
			...init,
			headers: {
				'Content-Type': 'application/json',
				...init?.headers
			}
		});

		if (!response.ok) {
			const body = await response.json().catch(() => ({}));
			throw new Error((body as { error?: string }).error ?? `HTTP ${response.status}`);
		}

		return response.json() as Promise<T>;
	}

	// ===== Lifecycle =====

	destroy(): void {
		if (this.eventSource) {
			this.eventSource.close();
			this.eventSource = null;
		}
		this._initialized = false;
	}
}

export const youtubeService = new YouTubeService();
