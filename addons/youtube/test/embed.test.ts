import { describe, it, expect } from 'vitest';
import {
	isValidYouTubeId,
	extractYouTubeId,
	getEmbedUrl,
	getThumbnailUrl,
	getWatchUrl,
	getVideoInfo
} from '../src/embed';

describe('isValidYouTubeId', () => {
	it('accepts valid 11-character IDs', () => {
		expect(isValidYouTubeId('dQw4w9WgXcQ')).toBe(true);
		expect(isValidYouTubeId('abc-_123ABC')).toBe(true);
	});

	it('rejects empty string', () => {
		expect(isValidYouTubeId('')).toBe(false);
	});

	it('rejects IDs that are too short', () => {
		expect(isValidYouTubeId('tooshort')).toBe(false);
	});

	it('rejects IDs that are too long', () => {
		expect(isValidYouTubeId('toolongstring!')).toBe(false);
	});

	it('rejects IDs with spaces', () => {
		expect(isValidYouTubeId('abc def ghij')).toBe(false);
	});

	it('rejects IDs with special characters', () => {
		expect(isValidYouTubeId('abc!@#$%^&()')).toBe(false);
	});
});

describe('extractYouTubeId', () => {
	it('extracts from bare ID', () => {
		expect(extractYouTubeId('dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
	});

	it('extracts from bare ID with whitespace', () => {
		expect(extractYouTubeId('  dQw4w9WgXcQ  ')).toBe('dQw4w9WgXcQ');
	});

	it('extracts from watch URL', () => {
		expect(extractYouTubeId('https://www.youtube.com/watch?v=dQw4w9WgXcQ')).toBe(
			'dQw4w9WgXcQ'
		);
	});

	it('extracts from short URL', () => {
		expect(extractYouTubeId('https://youtu.be/dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
	});

	it('extracts from embed URL', () => {
		expect(extractYouTubeId('https://www.youtube.com/embed/dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
	});

	it('extracts from shorts URL', () => {
		expect(extractYouTubeId('https://www.youtube.com/shorts/dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
	});

	it('extracts from v/ URL', () => {
		expect(extractYouTubeId('https://www.youtube.com/v/dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
	});

	it('returns null for invalid input', () => {
		expect(extractYouTubeId('not a url')).toBeNull();
	});

	it('returns null for empty string', () => {
		expect(extractYouTubeId('')).toBeNull();
	});

	it('returns null for unrelated URL', () => {
		expect(extractYouTubeId('https://example.com/video')).toBeNull();
	});
});

describe('getEmbedUrl', () => {
	it('generates embed URL without autoplay', () => {
		expect(getEmbedUrl('dQw4w9WgXcQ')).toBe(
			'https://www.youtube.com/embed/dQw4w9WgXcQ?rel=0'
		);
	});

	it('generates embed URL with autoplay', () => {
		const url = getEmbedUrl('dQw4w9WgXcQ', true);
		expect(url).toContain('autoplay=1');
		expect(url).toContain('rel=0');
		expect(url).toContain('embed/dQw4w9WgXcQ');
	});
});

describe('getThumbnailUrl', () => {
	it('generates thumbnail URL with default quality', () => {
		expect(getThumbnailUrl('dQw4w9WgXcQ')).toBe(
			'https://img.youtube.com/vi/dQw4w9WgXcQ/hqdefault.jpg'
		);
	});

	it('generates thumbnail URL with maxresdefault quality', () => {
		expect(getThumbnailUrl('dQw4w9WgXcQ', 'maxresdefault')).toBe(
			'https://img.youtube.com/vi/dQw4w9WgXcQ/maxresdefault.jpg'
		);
	});

	it('generates thumbnail URL with default quality variant', () => {
		expect(getThumbnailUrl('dQw4w9WgXcQ', 'default')).toBe(
			'https://img.youtube.com/vi/dQw4w9WgXcQ/default.jpg'
		);
	});
});

describe('getWatchUrl', () => {
	it('generates watch URL', () => {
		expect(getWatchUrl('dQw4w9WgXcQ')).toBe('https://www.youtube.com/watch?v=dQw4w9WgXcQ');
	});
});

describe('getVideoInfo', () => {
	it('returns full video info object', () => {
		const info = getVideoInfo('dQw4w9WgXcQ');
		expect(info.videoId).toBe('dQw4w9WgXcQ');
		expect(info.embedUrl).toContain('embed/dQw4w9WgXcQ');
		expect(info.thumbnailUrl).toContain('dQw4w9WgXcQ');
		expect(info.watchUrl).toContain('v=dQw4w9WgXcQ');
	});

	it('returns correct types', () => {
		const info = getVideoInfo('dQw4w9WgXcQ');
		expect(typeof info.videoId).toBe('string');
		expect(typeof info.embedUrl).toBe('string');
		expect(typeof info.thumbnailUrl).toBe('string');
		expect(typeof info.watchUrl).toBe('string');
	});
});
