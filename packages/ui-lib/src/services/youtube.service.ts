import { writable, get, type Writable } from 'svelte/store';
import { browser } from '$app/environment';
import { fetchJson, subscribeSSE } from 'ui-lib/transport/fetch-helpers';
import type { TransportEventSource } from 'ui-lib/transport/transport.type';
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
	type MediaMode,
	type VideoQuality,
	type VideoFormat,
	type SubtitleMode,
	type YouTubeStreamUrlResult,
	type YouTubeStreamFormat
} from 'ui-lib/types/youtube.type';

const API_PREFIX = '/api/ytdl';

// Default settings (used before server fetch completes)
const initialSettings: YouTubeSettings = {
	id: 'youtube-settings',
	downloadMode: 'audio',
	defaultQuality: 'high',
	defaultFormat: 'aac',
	defaultVideoQuality: 'best',
	defaultVideoFormat: 'mp4',
	subtitleMode: 'none',
	subtitleLangs: [],
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

	private eventSource: TransportEventSource | null = null;
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
				fetchJson<YouTubeManagerStats>('/api/ytdl/status'),
				fetchJson<DownloaderStatus>('/api/ytdl/ytdlp/status'),
				fetchJson<Omit<YouTubeSettings, 'id'>>('/api/ytdl/settings')
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
			const info = await fetchJson<YouTubeVideoInfo>(
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
			const info = await fetchJson<YouTubePlaylistInfo>(
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

			if (settings.subtitleMode !== 'none') {
				body.subtitleMode = settings.subtitleMode;
				body.subtitleLangs = settings.subtitleLangs;
			}

			const result = await fetchJson<{ downloadId: string }>('/api/ytdl/downloads', {
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
			await fetchJson(`/api/ytdl/downloads/${downloadId}`, { method: 'DELETE' });
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
			await fetchJson('/api/ytdl/downloads/completed', { method: 'DELETE' });
			// Refresh downloads list since SSE might not trigger for removed items
			const downloads = await fetchJson<YouTubeDownloadProgress[]>('/api/ytdl/downloads');
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

		if (settings.subtitleMode !== 'none') {
			body.subtitleMode = settings.subtitleMode;
			body.subtitleLangs = settings.subtitleLangs;
		}

		try {
			const result = await fetchJson<{ downloadIds: string[] }>('/api/ytdl/downloads/playlist', {
				method: 'POST',
				body: JSON.stringify(body)
			});

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

		if (settings.subtitleMode !== 'none') {
			body.subtitleMode = settings.subtitleMode;
			body.subtitleLangs = settings.subtitleLangs;
		}

		try {
			const result = await fetchJson<{ downloadId: string }>('/api/ytdl/downloads', {
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
			await fetchJson('/api/ytdl/downloads/queue', { method: 'DELETE' });
		} catch (error) {
			console.error('[YouTube] Failed to clear queue:', error);
		}
	}

	async queueDownloadWithMode(
		videoId: string,
		title: string,
		thumbnailUrl: string | null,
		mode: DownloadMode
	): Promise<string | null> {
		if (!browser) return null;

		const settings = this.get();
		const url = `https://www.youtube.com/watch?v=${videoId}`;

		const body: Record<string, unknown> = {
			url,
			videoId,
			title,
			mode,
			quality: settings.defaultQuality,
			format: settings.defaultFormat,
			thumbnailUrl,
			durationSeconds: null,
			channelName: null
		};

		if (mode === 'video' || mode === 'both') {
			body.videoQuality = settings.defaultVideoQuality;
			body.videoFormat = settings.defaultVideoFormat;
		}

		if (settings.subtitleMode !== 'none') {
			body.subtitleMode = settings.subtitleMode;
			body.subtitleLangs = settings.subtitleLangs;
		}

		try {
			const result = await fetchJson<{ downloadId: string }>('/api/ytdl/downloads', {
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

	// ===== Stream URL Extraction =====

	private streamUrlCache = new Map<string, { result: YouTubeStreamUrlResult; fetchedAt: number }>();

	async fetchStreamUrls(videoId: string): Promise<YouTubeStreamUrlResult | null> {
		if (!browser) return null;

		const now = Math.floor(Date.now() / 1000);
		const cached = this.streamUrlCache.get(videoId);
		if (cached && cached.result.expiresAt - 300 > now) {
			return cached.result;
		}

		try {
			const url = `https://www.youtube.com/watch?v=${videoId}`;
			const result = await fetchJson<YouTubeStreamUrlResult>(
				`/api/ytdl/info/stream-urls?url=${encodeURIComponent(url)}`
			);
			this.streamUrlCache.set(videoId, { result, fetchedAt: now });
			return result;
		} catch (error) {
			console.error('[YouTube] Failed to extract stream URLs:', error);
			return null;
		}
	}

	selectBestMuxedFormat(result: YouTubeStreamUrlResult): YouTubeStreamFormat | null {
		const muxed = result.formats.filter((f) => !f.isAudioOnly && !f.isVideoOnly);
		if (muxed.length === 0) return null;
		muxed.sort((a, b) => {
			const heightDiff = (b.height ?? 0) - (a.height ?? 0);
			if (heightDiff !== 0) return heightDiff;
			return b.bitrate - a.bitrate;
		});
		return muxed[0];
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
			await fetchJson('/api/ytdl/settings', {
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

	setMediaMode(mode: MediaMode): void {
		this.updateSettings({ mediaMode: mode });
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

	setSubtitleMode(mode: SubtitleMode): void {
		this.updateSettings({ subtitleMode: mode });
	}

	setSubtitleLangs(langs: string[]): void {
		this.updateSettings({ subtitleLangs: langs });
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
			await fetchJson('/api/ytdl/settings', {
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
			const status = await fetchJson<DownloaderStatus>('/api/ytdl/ytdlp/status');
			this.state.update((s) => ({ ...s, downloaderStatus: status }));
		} catch {
			// ignore
		}
	}

	// ===== SSE Connection =====

	private connectSSE(): void {
		if (!browser) return;

		this.eventSource = subscribeSSE(`${API_PREFIX}/downloads/events`);

		this.eventSource.addEventListener('progress', (data: string) => {
			try {
				const progress = JSON.parse(data) as YouTubeDownloadProgress;
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

		this.eventSource.addEventListener('stats', (data: string) => {
			try {
				const stats = JSON.parse(data) as YouTubeManagerStats;
				this.state.update((s) => ({ ...s, stats }));
			} catch {
				// ignore parse errors
			}
		});
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
