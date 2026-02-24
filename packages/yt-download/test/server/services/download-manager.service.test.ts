import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { DownloadManagerService } from '../../../src/server/services/download-manager.service.js';
import { SSEBroadcasterService } from '../../../src/server/services/sse-broadcaster.service.js';

// Minimal stub that satisfies what DownloadManagerService needs from YtDlpService
function createMockYtDlp() {
	return {
		isAvailable: () => true,
		getBinaryPath: () => '/usr/local/bin/yt-dlp',
		initialize: () => {},
		downloadBinary: async () => '/usr/local/bin/yt-dlp',
		getVersion: async () => '2024.01.01',
		getVideoInfo: async () => ({ title: 'Test', duration: 100, thumbnailUrl: null, uploader: null, videoId: 'test' }),
		getPlaylistInfo: async () => ({ playlistId: 'test', title: 'Test', videoCount: 0, videos: [], thumbnailUrl: null, author: null }),
		spawnDownload: () => {
			throw new Error('Not implemented in test');
		}
	} as any;
}

describe('DownloadManagerService', () => {
	let manager: DownloadManagerService;
	let broadcaster: SSEBroadcasterService;

	beforeEach(() => {
		broadcaster = new SSEBroadcasterService();
		manager = new DownloadManagerService(createMockYtDlp(), broadcaster);
	});

	afterEach(() => {
		broadcaster.destroy();
	});

	it('should return empty stats initially', () => {
		const stats = manager.getStats();
		expect(stats.activeDownloads).toBe(0);
		expect(stats.queuedDownloads).toBe(0);
		expect(stats.completedDownloads).toBe(0);
		expect(stats.failedDownloads).toBe(0);
	});

	it('should return default config', () => {
		const config = manager.getConfig();
		expect(config.defaultQuality).toBe('high');
		expect(config.defaultFormat).toBe('aac');
		expect(config.poToken).toBeNull();
		expect(config.cookies).toBeNull();
	});

	it('should update config', () => {
		manager.updateConfig({ defaultQuality: 'best', poToken: 'test-token' });
		const config = manager.getConfig();
		expect(config.defaultQuality).toBe('best');
		expect(config.poToken).toBe('test-token');
		expect(config.defaultFormat).toBe('aac'); // unchanged
	});

	it('should generate unique download IDs', () => {
		const id1 = manager.queueDownload({
			url: 'https://youtube.com/watch?v=test1',
			videoId: 'test1',
			title: 'Test 1'
		});
		const id2 = manager.queueDownload({
			url: 'https://youtube.com/watch?v=test2',
			videoId: 'test2',
			title: 'Test 2'
		});

		expect(id1).not.toBe(id2);
		expect(id1).toMatch(/^yt-\d+-\d+$/);
		expect(id2).toMatch(/^yt-\d+-\d+$/);
	});

	it('should track queued downloads', () => {
		manager.queueDownload({
			url: 'https://youtube.com/watch?v=test1',
			videoId: 'test1',
			title: 'Test 1'
		});

		const all = manager.getAllProgress();
		expect(all).toHaveLength(1);
		expect(all[0].title).toBe('Test 1');
		// The queue processor starts immediately and the mock throws,
		// so the download transitions to failed quickly
		expect(['pending', 'fetching', 'failed']).toContain(all[0].state);
	});

	it('should queue playlist downloads', () => {
		const ids = manager.queuePlaylistDownloads({
			videos: [
				{ url: 'https://youtube.com/watch?v=a', videoId: 'a', title: 'Video A' },
				{ url: 'https://youtube.com/watch?v=b', videoId: 'b', title: 'Video B' },
				{ url: 'https://youtube.com/watch?v=c', videoId: 'c', title: 'Video C' }
			]
		});

		expect(ids).toHaveLength(3);
		expect(manager.getAllProgress()).toHaveLength(3);
	});

	it('should get progress for a specific download', () => {
		const id = manager.queueDownload({
			url: 'https://youtube.com/watch?v=test1',
			videoId: 'test1',
			title: 'Test 1'
		});

		const progress = manager.getProgress(id);
		expect(progress).not.toBeNull();
		expect(progress!.downloadId).toBe(id);
	});

	it('should return null for unknown download ID', () => {
		expect(manager.getProgress('nonexistent')).toBeNull();
	});

	it('should cancel a pending download', () => {
		const id = manager.queueDownload({
			url: 'https://youtube.com/watch?v=test1',
			videoId: 'test1',
			title: 'Test 1'
		});

		manager.cancelDownload(id);

		const progress = manager.getProgress(id);
		expect(progress!.state).toBe('cancelled');
	});

	it('should clear completed downloads', () => {
		const id = manager.queueDownload({
			url: 'https://youtube.com/watch?v=test1',
			videoId: 'test1',
			title: 'Test 1'
		});

		// Cancel to get into a clearable state
		manager.cancelDownload(id);
		expect(manager.getAllProgress()).toHaveLength(1);

		manager.clearCompleted();
		expect(manager.getAllProgress()).toHaveLength(0);
	});

	it('should clear the queue', () => {
		manager.queueDownload({
			url: 'https://youtube.com/watch?v=test1',
			videoId: 'test1',
			title: 'Test 1'
		});
		manager.queueDownload({
			url: 'https://youtube.com/watch?v=test2',
			videoId: 'test2',
			title: 'Test 2'
		});

		manager.clearQueue();

		const all = manager.getAllProgress();
		// Downloads may be cancelled or failed (queue processor may have already
		// started processing them before clearQueue was called)
		for (const dl of all) {
			expect(['cancelled', 'failed']).toContain(dl.state);
		}
	});

	it('should use custom quality and format when specified', () => {
		const id = manager.queueDownload({
			url: 'https://youtube.com/watch?v=test1',
			videoId: 'test1',
			title: 'Test 1',
			quality: 'best',
			format: 'opus'
		});

		const progress = manager.getProgress(id);
		expect(progress!.quality).toBe('best');
		expect(progress!.format).toBe('opus');
	});

	it('should use default quality and format when not specified', () => {
		const id = manager.queueDownload({
			url: 'https://youtube.com/watch?v=test1',
			videoId: 'test1',
			title: 'Test 1'
		});

		const progress = manager.getProgress(id);
		expect(progress!.quality).toBe('high');
		expect(progress!.format).toBe('aac');
	});
});
