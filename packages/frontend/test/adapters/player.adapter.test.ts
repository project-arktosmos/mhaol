import { describe, it, expect } from 'vitest';
import { playerAdapter } from '../../src/adapters/classes/player.adapter';

describe('PlayerAdapter', () => {
	describe('formatDuration', () => {
		it('returns -- for null', () => {
			expect(playerAdapter.formatDuration(null)).toBe('--');
		});

		it('returns -- for negative', () => {
			expect(playerAdapter.formatDuration(-1)).toBe('--');
		});

		it('formats seconds only', () => {
			expect(playerAdapter.formatDuration(45)).toBe('0:45');
		});

		it('formats minutes and seconds', () => {
			expect(playerAdapter.formatDuration(125)).toBe('2:05');
		});

		it('formats hours', () => {
			expect(playerAdapter.formatDuration(3661)).toBe('1:01:01');
		});

		it('pads zeros', () => {
			expect(playerAdapter.formatDuration(3600)).toBe('1:00:00');
		});
	});

	describe('formatSize', () => {
		it('formats zero bytes', () => {
			expect(playerAdapter.formatSize(0)).toBe('0 B');
		});

		it('formats gigabytes', () => {
			expect(playerAdapter.formatSize(1073741824)).toBe('1.0 GB');
		});
	});

	describe('getFormatLabel', () => {
		it('uses videoFormat when available', () => {
			expect(playerAdapter.getFormatLabel({ videoFormat: 'mp4' } as any)).toBe('MP4');
		});

		it('falls back to format', () => {
			expect(playerAdapter.getFormatLabel({ format: 'webm' } as any)).toBe('WEBM');
		});

		it('returns Unknown when no format', () => {
			expect(playerAdapter.getFormatLabel({} as any)).toBe('Unknown');
		});
	});

	describe('getSourceBadgeClass', () => {
		it('maps source types to badge classes', () => {
			expect(playerAdapter.getSourceBadgeClass('youtube')).toBe('badge-secondary');
			expect(playerAdapter.getSourceBadgeClass('torrent')).toBe('badge-accent');
			expect(playerAdapter.getSourceBadgeClass('library')).toBe('badge-info');
		});
	});

	describe('fromMediaItem', () => {
		it('converts a MediaItem to PlayableFile', () => {
			const item = {
				id: '1',
				name: 'test.mp4',
				path: '/path/to/test.mp4',
				mediaTypeId: 'video',
				extension: 'mp4',
				createdAt: '2023-01-01'
			};
			const result = playerAdapter.fromMediaItem(item as any);
			expect(result.id).toBe('1');
			expect(result.type).toBe('library');
			expect(result.mode).toBe('video');
			expect(result.format).toBe('mp4');
		});

		it('sets mode to audio for audio mediaType', () => {
			const item = {
				id: '2',
				name: 'song.mp3',
				path: '/path/to/song.mp3',
				mediaTypeId: 'audio',
				extension: 'mp3',
				createdAt: '2023-01-01'
			};
			const result = playerAdapter.fromMediaItem(item as any);
			expect(result.mode).toBe('audio');
		});
	});
});
