import { describe, it, expect } from 'vitest';
import { youTubeCardAdapter } from '../../src/adapters/classes/youtube-card.adapter';

describe('YouTubeCardAdapter', () => {
	describe('fromContent', () => {
		it('transforms YouTubeContent to LibraryCardItem', () => {
			const content = {
				youtubeId: 'abc123',
				title: 'Test Video',
				thumbnailUrl: 'https://img.youtube.com/vi/abc123/hqdefault.jpg',
				durationSeconds: 300,
				channelName: 'Test Channel',
				hasVideo: true,
				hasAudio: true
			};
			const result = youTubeCardAdapter.fromContent(content as any);
			expect(result.videoId).toBe('abc123');
			expect(result.title).toBe('Test Video');
			expect(result.durationSeconds).toBe(300);
			expect(result.channelName).toBe('Test Channel');
			expect(result.hasVideo).toBe(true);
		});
	});

	describe('fromRssVideo', () => {
		it('transforms YouTubeRssVideo to LibraryCardItem', () => {
			const video = {
				videoId: 'xyz789',
				title: 'RSS Video',
				thumbnail: 'https://img.youtube.com/vi/xyz789/hqdefault.jpg'
			};
			const result = youTubeCardAdapter.fromRssVideo(video as any);
			expect(result.videoId).toBe('xyz789');
			expect(result.title).toBe('RSS Video');
			expect(result.durationSeconds).toBeNull();
			expect(result.channelName).toBeNull();
			expect(result.hasVideo).toBe(false);
		});
	});

	describe('fromSearchItem', () => {
		it('transforms YouTubeSearchItem to LibraryCardItem', () => {
			const item = {
				videoId: 'search1',
				title: 'Search Result',
				thumbnail: 'https://img.youtube.com/vi/search1/hqdefault.jpg',
				duration: 120,
				uploaderName: 'Uploader'
			};
			const result = youTubeCardAdapter.fromSearchItem(item as any);
			expect(result.videoId).toBe('search1');
			expect(result.durationSeconds).toBe(120);
			expect(result.channelName).toBe('Uploader');
		});

		it('sets null duration for zero', () => {
			const item = {
				videoId: 'search2',
				title: 'Live Stream',
				thumbnail: 'thumb.jpg',
				duration: 0,
				uploaderName: ''
			};
			const result = youTubeCardAdapter.fromSearchItem(item as any);
			expect(result.durationSeconds).toBeNull();
			expect(result.channelName).toBeNull();
		});
	});
});
