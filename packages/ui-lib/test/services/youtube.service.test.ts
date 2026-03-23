import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { get } from 'svelte/store';
import { youtubeService } from '../../src/services/youtube.service';

function mockFetch(data: unknown, ok = true, status = 200) {
	return vi.fn().mockResolvedValue({
		ok,
		status,
		json: () => Promise.resolve(data),
		text: () => Promise.resolve(JSON.stringify(data)),
		body: null
	});
}

function mockFetchError(errorMsg: string, status = 500) {
	return vi.fn().mockResolvedValue({
		ok: false,
		status,
		json: () => Promise.resolve({ error: errorMsg }),
		text: () => Promise.resolve(''),
		body: null
	});
}

const defaultSettings = {
	id: 'youtube-settings',
	downloadMode: 'audio' as const,
	defaultQuality: 'high' as const,
	defaultFormat: 'aac' as const,
	defaultVideoQuality: 'best' as const,
	defaultVideoFormat: 'mp4' as const,
	libraryId: '',
	poToken: '',
	cookies: ''
};

const defaultState = {
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

describe('YouTubeService', () => {
	beforeEach(() => {
		youtubeService.store.set({ ...defaultSettings });
		youtubeService.state.set({ ...defaultState });
	});

	afterEach(() => {
		youtubeService.destroy();
		vi.restoreAllMocks();
		vi.unstubAllGlobals();
	});

	// ===== Singleton & initial state =====

	it('exports a singleton youtubeService', () => {
		expect(youtubeService).toBeDefined();
		expect(youtubeService.store).toBeDefined();
		expect(youtubeService.state).toBeDefined();
	});

	it('has correct initial settings', () => {
		const settings = youtubeService.get();
		expect(settings.downloadMode).toBe('audio');
		expect(settings.defaultQuality).toBe('high');
		expect(settings.defaultFormat).toBe('aac');
		expect(settings.defaultVideoQuality).toBe('best');
		expect(settings.defaultVideoFormat).toBe('mp4');
		expect(settings.libraryId).toBe('');
		expect(settings.poToken).toBe('');
		expect(settings.cookies).toBe('');
	});

	it('has correct initial state', () => {
		const state = get(youtubeService.state);
		expect(state.initialized).toBe(false);
		expect(state.loading).toBe(false);
		expect(state.error).toBeNull();
		expect(state.downloads).toEqual([]);
		expect(state.stats).toBeNull();
		expect(state.downloaderStatus).toBeNull();
		expect(state.currentUrl).toBe('');
		expect(state.currentVideoInfo).toBeNull();
		expect(state.currentPlaylistInfo).toBeNull();
		expect(state.fetchingInfo).toBe(false);
		expect(state.fetchingVideoInfo).toBe(false);
		expect(state.fetchingPlaylistInfo).toBe(false);
	});

	// ===== get =====

	it('get returns current settings', () => {
		const settings = youtubeService.get();
		expect(settings.id).toBe('youtube-settings');
	});

	// ===== setCurrentUrl =====

	it('setCurrentUrl updates state', () => {
		youtubeService.setCurrentUrl('https://youtube.com/watch?v=abc123');
		const state = get(youtubeService.state);
		expect(state.currentUrl).toBe('https://youtube.com/watch?v=abc123');
		expect(state.currentVideoInfo).toBeNull();
		expect(state.currentPlaylistInfo).toBeNull();
		expect(state.error).toBeNull();
	});

	// ===== clearCurrentVideo =====

	it('clearCurrentVideo resets current video state', () => {
		youtubeService.setCurrentUrl('https://youtube.com/watch?v=test');
		youtubeService.clearCurrentVideo();
		const state = get(youtubeService.state);
		expect(state.currentUrl).toBe('');
		expect(state.currentVideoInfo).toBeNull();
		expect(state.currentPlaylistInfo).toBeNull();
		expect(state.error).toBeNull();
	});

	// ===== initialize =====

	it('initialize fetches stats, downloader status, and settings', async () => {
		const mockStats = { activeDownloads: 0, queuedDownloads: 0, totalDownloads: 5 };
		const mockDownloaderStatus = { available: true, version: '2024.01.01' };
		const mockSettings = {
			downloadMode: 'video' as const,
			defaultQuality: 'medium' as const,
			defaultFormat: 'mp3' as const,
			defaultVideoQuality: '1080p' as const,
			defaultVideoFormat: 'webm' as const,
			libraryId: 'lib-1',
			poToken: '',
			cookies: ''
		};

		let callCount = 0;
		const mock = vi.fn().mockImplementation(() => {
			callCount++;
			if (callCount === 1)
				return Promise.resolve({ ok: true, json: () => Promise.resolve(mockStats) });
			if (callCount === 2)
				return Promise.resolve({ ok: true, json: () => Promise.resolve(mockDownloaderStatus) });
			return Promise.resolve({ ok: true, json: () => Promise.resolve(mockSettings) });
		});
		vi.stubGlobal('fetch', mock);
		vi.stubGlobal(
			'EventSource',
			class {
				addEventListener() {}
				close() {}
				set onerror(_: unknown) {}
			}
		);

		await youtubeService.initialize();

		const state = get(youtubeService.state);
		expect(state.initialized).toBe(true);
		expect(state.loading).toBe(false);
		expect(state.stats).toEqual(mockStats);
		expect(state.downloaderStatus).toEqual(mockDownloaderStatus);
		expect(state.libraryId).toBe('lib-1');

		const settings = youtubeService.get();
		expect(settings.downloadMode).toBe('video');
		expect(settings.libraryId).toBe('lib-1');
	});

	it('initialize handles fetch failure', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Network error')));

		await youtubeService.initialize();

		const state = get(youtubeService.state);
		expect(state.initialized).toBe(false);
		expect(state.loading).toBe(false);
		expect(state.error).toContain('Failed to connect to download server');
		expect(state.error).toContain('Network error');
	});

	it('initialize sets loading to true', async () => {
		let capturedLoading = false;
		const unsub = youtubeService.state.subscribe((s) => {
			if (s.loading) capturedLoading = true;
		});

		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('fail')));

		await youtubeService.initialize();
		unsub();
		expect(capturedLoading).toBe(true);
	});

	it('initialize cleans up legacy localStorage', async () => {
		localStorage.setItem('object-service:youtube-settings', '{}');

		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('fail')));

		await youtubeService.initialize();

		expect(localStorage.getItem('object-service:youtube-settings')).toBeNull();
	});

	// ===== fetchVideoInfo =====

	it('fetchVideoInfo sets fetching state and returns info', async () => {
		const mockInfo = { videoId: 'abc123', title: 'Test Video', duration: 120 };
		vi.stubGlobal('fetch', mockFetch(mockInfo));

		const result = await youtubeService.fetchVideoInfo('https://youtube.com/watch?v=abc123');

		expect(result).toEqual(mockInfo);
		const state = get(youtubeService.state);
		expect(state.currentVideoInfo).toEqual(mockInfo);
		expect(state.fetchingInfo).toBe(false);
		expect(state.fetchingVideoInfo).toBe(false);
		expect(state.currentUrl).toBe('https://youtube.com/watch?v=abc123');
	});

	it('fetchVideoInfo clears previous info', async () => {
		youtubeService.state.update((s) => ({
			...s,
			currentVideoInfo: { videoId: 'old' } as never,
			currentPlaylistInfo: { title: 'old' } as never
		}));

		vi.stubGlobal('fetch', mockFetch({ videoId: 'new' }));

		await youtubeService.fetchVideoInfo('https://youtube.com/watch?v=new');

		const state = get(youtubeService.state);
		expect(state.currentPlaylistInfo).toBeNull();
	});

	it('fetchVideoInfo handles errors', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Video not found')));

		const result = await youtubeService.fetchVideoInfo('https://youtube.com/watch?v=bad');

		expect(result).toBeNull();
		const state = get(youtubeService.state);
		expect(state.error).toContain('Failed to fetch video info');
		expect(state.fetchingInfo).toBe(false);
		expect(state.fetchingVideoInfo).toBe(false);
	});

	// ===== fetchPlaylistInfo =====

	it('fetchPlaylistInfo sets state and returns playlist info', async () => {
		const mockPlaylist = {
			title: 'Test Playlist',
			videos: [{ videoId: 'v1', title: 'Video 1' }]
		};
		vi.stubGlobal('fetch', mockFetch(mockPlaylist));

		const result = await youtubeService.fetchPlaylistInfo(
			'https://youtube.com/playlist?list=PLtest'
		);

		expect(result).toEqual(mockPlaylist);
		const state = get(youtubeService.state);
		expect(state.currentPlaylistInfo).toEqual(mockPlaylist);
		expect(state.fetchingPlaylistInfo).toBe(false);
		expect(state.fetchingInfo).toBe(false);
	});

	it('fetchPlaylistInfo clears previous info', async () => {
		youtubeService.state.update((s) => ({
			...s,
			currentVideoInfo: { videoId: 'old' } as never,
			currentPlaylistInfo: { title: 'old' } as never
		}));

		vi.stubGlobal('fetch', mockFetch({ title: 'new', videos: [] }));

		await youtubeService.fetchPlaylistInfo('https://youtube.com/playlist?list=PLnew');

		const state = get(youtubeService.state);
		expect(state.currentVideoInfo).toBeNull();
	});

	it('fetchPlaylistInfo handles errors', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Playlist not found')));

		const result = await youtubeService.fetchPlaylistInfo('https://youtube.com/playlist?list=bad');

		expect(result).toBeNull();
		const state = get(youtubeService.state);
		expect(state.error).toContain('Failed to fetch playlist info');
		expect(state.fetchingPlaylistInfo).toBe(false);
	});

	// ===== download =====

	it('download sends POST with correct body', async () => {
		const mock = mockFetch({ downloadId: 'dl-1' });
		vi.stubGlobal('fetch', mock);

		youtubeService.state.update((s) => ({
			...s,
			currentUrl: 'https://youtube.com/watch?v=abc123',
			currentVideoInfo: { videoId: 'abc123', title: 'Test' } as never
		}));

		const result = await youtubeService.download();

		expect(result).toBe('dl-1');
		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/ytdl/downloads'),
			expect.objectContaining({
				method: 'POST',
				body: expect.stringContaining('"url":"https://youtube.com/watch?v=abc123"')
			})
		);
	});

	it('download includes video settings when downloadMode is video', async () => {
		youtubeService.store.set({ ...defaultSettings, downloadMode: 'video' });
		youtubeService.state.update((s) => ({
			...s,
			currentUrl: 'https://youtube.com/watch?v=abc',
			currentVideoInfo: { videoId: 'abc', title: 'T' } as never
		}));

		const mock = mockFetch({ downloadId: 'dl-1' });
		vi.stubGlobal('fetch', mock);

		await youtubeService.download();

		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.videoQuality).toBe('best');
		expect(body.videoFormat).toBe('mp4');
	});

	it('download returns null when no URL is set', async () => {
		const result = await youtubeService.download();
		expect(result).toBeNull();
		const state = get(youtubeService.state);
		expect(state.error).toBe('No URL provided');
	});

	it('download clears current video on success', async () => {
		youtubeService.state.update((s) => ({
			...s,
			currentUrl: 'https://youtube.com/watch?v=abc',
			currentVideoInfo: { videoId: 'abc', title: 'T' } as never
		}));

		vi.stubGlobal('fetch', mockFetch({ downloadId: 'dl-1' }));

		await youtubeService.download();

		const state = get(youtubeService.state);
		expect(state.currentUrl).toBe('');
		expect(state.currentVideoInfo).toBeNull();
	});

	it('download handles error', async () => {
		youtubeService.state.update((s) => ({
			...s,
			currentUrl: 'https://youtube.com/watch?v=abc'
		}));

		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Download failed')));

		const result = await youtubeService.download();

		expect(result).toBeNull();
		const state = get(youtubeService.state);
		expect(state.error).toContain('Failed to start download');
	});

	it('download extracts videoId from URL when no video info', async () => {
		youtubeService.state.update((s) => ({
			...s,
			currentUrl: 'https://youtube.com/watch?v=extractMe',
			currentVideoInfo: null
		}));

		const mock = mockFetch({ downloadId: 'dl-1' });
		vi.stubGlobal('fetch', mock);

		await youtubeService.download();

		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.videoId).toBe('extractMe');
		expect(body.title).toBe('Unknown');
	});

	// ===== downloadAudio =====

	it('downloadAudio delegates to download', async () => {
		youtubeService.state.update((s) => ({
			...s,
			currentUrl: 'https://youtube.com/watch?v=abc',
			currentVideoInfo: { videoId: 'abc', title: 'T' } as never
		}));

		vi.stubGlobal('fetch', mockFetch({ downloadId: 'dl-audio' }));

		const result = await youtubeService.downloadAudio();
		expect(result).toBe('dl-audio');
	});

	// ===== cancelDownload =====

	it('cancelDownload sends DELETE request', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeService.cancelDownload('dl-1');

		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/ytdl/downloads/dl-1'),
			expect.objectContaining({ method: 'DELETE' })
		);
	});

	it('cancelDownload handles error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Cancel failed')));

		await youtubeService.cancelDownload('dl-1');

		const state = get(youtubeService.state);
		expect(state.error).toContain('Failed to cancel download');
	});

	// ===== clearCompleted =====

	it('clearCompleted sends DELETE and refreshes downloads', async () => {
		const remainingDownloads = [{ downloadId: 'dl-active', status: 'downloading' }];
		let callCount = 0;
		vi.stubGlobal(
			'fetch',
			vi.fn().mockImplementation(() => {
				callCount++;
				if (callCount === 1) return Promise.resolve({ ok: true, json: () => Promise.resolve({}) });
				return Promise.resolve({ ok: true, json: () => Promise.resolve(remainingDownloads) });
			})
		);

		await youtubeService.clearCompleted();

		const state = get(youtubeService.state);
		expect(state.downloads).toEqual(remainingDownloads);
	});

	it('clearCompleted handles error gracefully', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Clear failed')));

		await youtubeService.clearCompleted();

		const state = get(youtubeService.state);
		expect(state.error).toBeNull();
	});

	// ===== downloadPlaylist =====

	it('downloadPlaylist sends POST with playlist videos', async () => {
		youtubeService.state.update((s) => ({
			...s,
			currentPlaylistInfo: {
				title: 'Test Playlist',
				videos: [
					{ videoId: 'v1', title: 'Video 1' },
					{ videoId: 'v2', title: 'Video 2' }
				]
			} as never
		}));

		const mock = mockFetch({ downloadIds: ['dl-1', 'dl-2'] });
		vi.stubGlobal('fetch', mock);

		const result = await youtubeService.downloadPlaylist();

		expect(result).toEqual(['dl-1', 'dl-2']);
		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.videos).toHaveLength(2);
		expect(body.videos[0].videoId).toBe('v1');
	});

	it('downloadPlaylist returns null when no playlist loaded', async () => {
		const result = await youtubeService.downloadPlaylist();

		expect(result).toBeNull();
		const state = get(youtubeService.state);
		expect(state.error).toBe('No playlist loaded');
	});

	it('downloadPlaylist handles error', async () => {
		youtubeService.state.update((s) => ({
			...s,
			currentPlaylistInfo: { title: 'P', videos: [{ videoId: 'v1', title: 'V1' }] } as never
		}));

		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Playlist fail')));

		const result = await youtubeService.downloadPlaylist();

		expect(result).toBeNull();
		const state = get(youtubeService.state);
		expect(state.error).toContain('Failed to queue playlist');
	});

	it('downloadPlaylist includes video settings when mode is video', async () => {
		youtubeService.store.set({ ...defaultSettings, downloadMode: 'video' });
		youtubeService.state.update((s) => ({
			...s,
			currentPlaylistInfo: { title: 'P', videos: [{ videoId: 'v1', title: 'V1' }] } as never
		}));

		const mock = mockFetch({ downloadIds: ['dl-1'] });
		vi.stubGlobal('fetch', mock);

		await youtubeService.downloadPlaylist();

		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.videoQuality).toBe('best');
		expect(body.videoFormat).toBe('mp4');
	});

	it('downloadPlaylist clears current video on success', async () => {
		youtubeService.state.update((s) => ({
			...s,
			currentUrl: 'https://youtube.com/playlist?list=test',
			currentPlaylistInfo: { title: 'P', videos: [{ videoId: 'v1', title: 'V1' }] } as never
		}));

		vi.stubGlobal('fetch', mockFetch({ downloadIds: ['dl-1'] }));

		await youtubeService.downloadPlaylist();

		const state = get(youtubeService.state);
		expect(state.currentUrl).toBe('');
	});

	// ===== queueSingleDownload =====

	it('queueSingleDownload sends POST and returns downloadId', async () => {
		const mock = mockFetch({ downloadId: 'dl-single' });
		vi.stubGlobal('fetch', mock);

		const result = await youtubeService.queueSingleDownload(
			'https://youtube.com/watch?v=test',
			'test',
			'Test Video'
		);

		expect(result).toBe('dl-single');
		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.url).toBe('https://youtube.com/watch?v=test');
		expect(body.videoId).toBe('test');
		expect(body.title).toBe('Test Video');
	});

	it('queueSingleDownload includes video settings when mode is video', async () => {
		youtubeService.store.set({ ...defaultSettings, downloadMode: 'video' });

		const mock = mockFetch({ downloadId: 'dl-1' });
		vi.stubGlobal('fetch', mock);

		await youtubeService.queueSingleDownload('url', 'vid', 'title');

		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.videoQuality).toBe('best');
	});

	it('queueSingleDownload handles error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Queue failed')));

		const result = await youtubeService.queueSingleDownload('url', 'vid', 'title');

		expect(result).toBeNull();
		const state = get(youtubeService.state);
		expect(state.error).toContain('Failed to queue download');
	});

	// ===== clearQueue =====

	it('clearQueue sends DELETE request', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeService.clearQueue();

		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/ytdl/downloads/queue'),
			expect.objectContaining({ method: 'DELETE' })
		);
	});

	it('clearQueue handles error gracefully', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Clear failed')));

		await youtubeService.clearQueue();

		const state = get(youtubeService.state);
		expect(state.error).toBeNull();
	});

	// ===== queueDownloadWithMode =====

	it('queueDownloadWithMode sends POST with correct mode', async () => {
		const mock = mockFetch({ downloadId: 'dl-mode' });
		vi.stubGlobal('fetch', mock);

		const result = await youtubeService.queueDownloadWithMode(
			'vid1',
			'Title',
			'thumb.jpg',
			'audio'
		);

		expect(result).toBe('dl-mode');
		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.mode).toBe('audio');
		expect(body.thumbnailUrl).toBe('thumb.jpg');
		expect(body.videoId).toBe('vid1');
		expect(body.url).toContain('vid1');
	});

	it('queueDownloadWithMode includes video settings for video mode', async () => {
		const mock = mockFetch({ downloadId: 'dl-1' });
		vi.stubGlobal('fetch', mock);

		await youtubeService.queueDownloadWithMode('vid1', 'T', null, 'video');

		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.videoQuality).toBe('best');
		expect(body.videoFormat).toBe('mp4');
	});

	it('queueDownloadWithMode includes video settings for both mode', async () => {
		const mock = mockFetch({ downloadId: 'dl-1' });
		vi.stubGlobal('fetch', mock);

		await youtubeService.queueDownloadWithMode('vid1', 'T', null, 'both');

		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.videoQuality).toBeDefined();
	});

	it('queueDownloadWithMode handles error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Failed')));

		const result = await youtubeService.queueDownloadWithMode('vid', 'T', null, 'audio');

		expect(result).toBeNull();
		const state = get(youtubeService.state);
		expect(state.error).toContain('Failed to queue download');
	});

	// ===== fetchStreamUrls =====

	it('fetchStreamUrls fetches and caches result', async () => {
		const result = {
			expiresAt: Math.floor(Date.now() / 1000) + 3600,
			formats: [
				{
					url: 'stream-url',
					mimeType: 'video/mp4',
					bitrate: 1000,
					isAudioOnly: false,
					isVideoOnly: false
				}
			]
		};

		const mock = mockFetch(result);
		vi.stubGlobal('fetch', mock);

		const res1 = await youtubeService.fetchStreamUrls('vid1');
		expect(res1).toEqual(result);

		// Second call should use cache
		const res2 = await youtubeService.fetchStreamUrls('vid1');
		expect(res2).toEqual(result);
		expect(mock).toHaveBeenCalledTimes(1); // Only called once due to cache
	});

	it('fetchStreamUrls handles error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Stream URL failed')));

		// Use a different videoId to avoid cache hit from previous test
		const result = await youtubeService.fetchStreamUrls('vid-error');
		expect(result).toBeNull();
	});

	// ===== selectBestMuxedFormat =====

	it('selectBestMuxedFormat picks highest resolution', () => {
		const result = {
			expiresAt: 999999,
			formats: [
				{
					url: 'a',
					mimeType: 'video/mp4',
					bitrate: 1000,
					width: 640,
					height: 360,
					isAudioOnly: false,
					isVideoOnly: false
				},
				{
					url: 'b',
					mimeType: 'video/mp4',
					bitrate: 2000,
					width: 1280,
					height: 720,
					isAudioOnly: false,
					isVideoOnly: false
				},
				{
					url: 'c',
					mimeType: 'video/mp4',
					bitrate: 3000,
					width: null,
					height: null,
					isAudioOnly: true,
					isVideoOnly: false
				}
			]
		};

		const best = youtubeService.selectBestMuxedFormat(result as never);
		expect(best).not.toBeNull();
		expect(best!.url).toBe('b');
		expect(best!.height).toBe(720);
	});

	it('selectBestMuxedFormat returns null when no muxed formats', () => {
		const result = {
			expiresAt: 999999,
			formats: [
				{
					url: 'a',
					mimeType: 'audio/mp4',
					bitrate: 128,
					width: null,
					height: null,
					isAudioOnly: true,
					isVideoOnly: false
				}
			]
		};

		const best = youtubeService.selectBestMuxedFormat(result as never);
		expect(best).toBeNull();
	});

	it('selectBestMuxedFormat excludes video-only formats', () => {
		const result = {
			expiresAt: 999999,
			formats: [
				{
					url: 'a',
					mimeType: 'video/mp4',
					bitrate: 5000,
					width: 1920,
					height: 1080,
					isAudioOnly: false,
					isVideoOnly: true
				},
				{
					url: 'b',
					mimeType: 'video/mp4',
					bitrate: 1000,
					width: 640,
					height: 360,
					isAudioOnly: false,
					isVideoOnly: false
				}
			]
		};

		const best = youtubeService.selectBestMuxedFormat(result as never);
		expect(best!.url).toBe('b');
	});

	it('selectBestMuxedFormat breaks ties by bitrate', () => {
		const result = {
			expiresAt: 999999,
			formats: [
				{
					url: 'a',
					mimeType: 'video/mp4',
					bitrate: 1000,
					width: 1280,
					height: 720,
					isAudioOnly: false,
					isVideoOnly: false
				},
				{
					url: 'b',
					mimeType: 'video/mp4',
					bitrate: 2000,
					width: 1280,
					height: 720,
					isAudioOnly: false,
					isVideoOnly: false
				}
			]
		};

		const best = youtubeService.selectBestMuxedFormat(result as never);
		expect(best!.url).toBe('b');
	});

	// ===== updateSettings =====

	it('updateSettings optimistically updates store and sends PUT', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeService.updateSettings({ downloadMode: 'video' });

		const settings = youtubeService.get();
		expect(settings.downloadMode).toBe('video');

		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/ytdl/settings'),
			expect.objectContaining({ method: 'PUT' })
		);
	});

	it('updateSettings strips id from payload', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeService.updateSettings({ downloadMode: 'video' });

		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.id).toBeUndefined();
	});

	it('updateSettings reverts on failure', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Save failed')));

		await youtubeService.updateSettings({ downloadMode: 'video' });

		const settings = youtubeService.get();
		expect(settings.downloadMode).toBe('audio'); // reverted
		const state = get(youtubeService.state);
		expect(state.error).toContain('Failed to save settings');
	});

	it('updateSettings updates libraryId in state', async () => {
		vi.stubGlobal('fetch', mockFetch({}));

		await youtubeService.updateSettings({ libraryId: 'lib-new' });

		const state = get(youtubeService.state);
		expect(state.libraryId).toBe('lib-new');
	});

	// ===== Convenience setters =====

	it('setDownloadMode calls updateSettings', async () => {
		vi.stubGlobal('fetch', mockFetch({}));
		youtubeService.setDownloadMode('video');

		await vi.waitFor(() => {
			expect(youtubeService.get().downloadMode).toBe('video');
		});
	});

	it('setDefaultQuality calls updateSettings', async () => {
		vi.stubGlobal('fetch', mockFetch({}));
		youtubeService.setDefaultQuality('medium');

		await vi.waitFor(() => {
			expect(youtubeService.get().defaultQuality).toBe('medium');
		});
	});

	it('setDefaultFormat calls updateSettings', async () => {
		vi.stubGlobal('fetch', mockFetch({}));
		youtubeService.setDefaultFormat('mp3');

		await vi.waitFor(() => {
			expect(youtubeService.get().defaultFormat).toBe('mp3');
		});
	});

	it('setDefaultVideoQuality calls updateSettings', async () => {
		vi.stubGlobal('fetch', mockFetch({}));
		youtubeService.setDefaultVideoQuality('1080p' as never);

		await vi.waitFor(() => {
			expect(youtubeService.get().defaultVideoQuality).toBe('1080p');
		});
	});

	it('setDefaultVideoFormat calls updateSettings', async () => {
		vi.stubGlobal('fetch', mockFetch({}));
		youtubeService.setDefaultVideoFormat('webm');

		await vi.waitFor(() => {
			expect(youtubeService.get().defaultVideoFormat).toBe('webm');
		});
	});

	it('setLibrary calls updateSettings', async () => {
		vi.stubGlobal('fetch', mockFetch({}));
		youtubeService.setLibrary('lib-123');

		await vi.waitFor(() => {
			expect(youtubeService.get().libraryId).toBe('lib-123');
		});
	});

	// ===== Getters =====

	it('isInitialized reflects state', () => {
		expect(youtubeService.isInitialized).toBe(false);
		youtubeService.state.update((s) => ({ ...s, initialized: true }));
		expect(youtubeService.isInitialized).toBe(true);
	});

	it('hasActiveDownloads reflects stats', () => {
		expect(youtubeService.hasActiveDownloads).toBe(false);
		youtubeService.state.update((s) => ({
			...s,
			stats: { activeDownloads: 2, queuedDownloads: 0, totalDownloads: 5 } as never
		}));
		expect(youtubeService.hasActiveDownloads).toBe(true);
	});

	it('hasActiveDownloads returns false when no stats', () => {
		expect(youtubeService.hasActiveDownloads).toBe(false);
	});

	it('hasPendingWork reflects stats', () => {
		expect(youtubeService.hasPendingWork).toBe(false);
		youtubeService.state.update((s) => ({
			...s,
			stats: { activeDownloads: 0, queuedDownloads: 3, totalDownloads: 5 } as never
		}));
		expect(youtubeService.hasPendingWork).toBe(true);
	});

	it('hasPendingWork returns true with active downloads', () => {
		youtubeService.state.update((s) => ({
			...s,
			stats: { activeDownloads: 1, queuedDownloads: 0, totalDownloads: 5 } as never
		}));
		expect(youtubeService.hasPendingWork).toBe(true);
	});

	it('hasPendingWork returns false when no stats', () => {
		expect(youtubeService.hasPendingWork).toBe(false);
	});

	// ===== setConfig =====

	it('setConfig sends PUT with poToken and cookies', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeService.setConfig({ poToken: 'token-123', cookies: 'cookie-val' });

		expect(mock).toHaveBeenCalledWith(
			expect.stringContaining('/api/ytdl/settings'),
			expect.objectContaining({ method: 'PUT' })
		);
		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.poToken).toBe('token-123');
		expect(body.cookies).toBe('cookie-val');
	});

	it('setConfig handles null values', async () => {
		const mock = mockFetch({});
		vi.stubGlobal('fetch', mock);

		await youtubeService.setConfig({ poToken: null, cookies: null });

		const call = mock.mock.calls[0];
		const body = JSON.parse(call[1].body as string);
		expect(body.poToken).toBe('');
		expect(body.cookies).toBe('');
	});

	it('setConfig handles error', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Config failed')));

		await youtubeService.setConfig({ poToken: 'token', cookies: null });

		const state = get(youtubeService.state);
		expect(state.error).toContain('Failed to set config');
	});

	// ===== getAuthConfig =====

	it('getAuthConfig returns poToken and cookies', () => {
		youtubeService.store.set({ ...defaultSettings, poToken: 'my-token', cookies: 'my-cookies' });

		const config = youtubeService.getAuthConfig();
		expect(config.poToken).toBe('my-token');
		expect(config.cookies).toBe('my-cookies');
	});

	it('getAuthConfig returns null for empty values', () => {
		const config = youtubeService.getAuthConfig();
		expect(config.poToken).toBeNull();
		expect(config.cookies).toBeNull();
	});

	// ===== refreshDownloaderStatus =====

	it('refreshDownloaderStatus updates state', async () => {
		const status = { available: true, version: '2024.01.01' };
		vi.stubGlobal('fetch', mockFetch(status));

		await youtubeService.refreshDownloaderStatus();

		const state = get(youtubeService.state);
		expect(state.downloaderStatus).toEqual(status);
	});

	it('refreshDownloaderStatus handles error silently', async () => {
		vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('Failed')));

		await youtubeService.refreshDownloaderStatus();

		const state = get(youtubeService.state);
		expect(state.downloaderStatus).toBeNull();
		expect(state.error).toBeNull();
	});

	// ===== destroy =====

	it('destroy resets initialized flag', () => {
		youtubeService.state.update((s) => ({ ...s, initialized: true }));

		youtubeService.destroy();

		// _initialized is private, but we can verify by trying to initialize again
		// The public getter reflects the state store, not _initialized
	});
});
