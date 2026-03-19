import { describe, it, expect } from 'vitest';
import { extractVideoId, extractPlaylistId, isPlaylistUrl } from '../../src/types/youtube.type';

describe('extractVideoId', () => {
	it('extracts from standard watch URL', () => {
		expect(extractVideoId('https://www.youtube.com/watch?v=dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
	});

	it('extracts from short URL', () => {
		expect(extractVideoId('https://youtu.be/dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
	});

	it('extracts from embed URL', () => {
		expect(extractVideoId('https://www.youtube.com/embed/dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
	});

	it('extracts from shorts URL', () => {
		expect(extractVideoId('https://www.youtube.com/shorts/dQw4w9WgXcQ')).toBe('dQw4w9WgXcQ');
	});

	it('extracts with additional query params', () => {
		expect(extractVideoId('https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLxyz&t=10')).toBe(
			'dQw4w9WgXcQ'
		);
	});

	it('returns null for non-YouTube URL', () => {
		expect(extractVideoId('https://example.com/video')).toBeNull();
	});

	it('returns null for playlist-only URL', () => {
		expect(extractVideoId('https://www.youtube.com/playlist?list=PLxyz')).toBeNull();
	});
});

describe('extractPlaylistId', () => {
	it('extracts playlist ID from URL', () => {
		expect(
			extractPlaylistId('https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf')
		).toBe('PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf');
	});

	it('extracts playlist ID from video URL with list param', () => {
		expect(extractPlaylistId('https://www.youtube.com/watch?v=abc123&list=PLxyz')).toBe('PLxyz');
	});

	it('returns null when no playlist ID', () => {
		expect(extractPlaylistId('https://www.youtube.com/watch?v=abc123')).toBeNull();
	});
});

describe('isPlaylistUrl', () => {
	it('returns true for playlist URL', () => {
		expect(isPlaylistUrl('https://www.youtube.com/playlist?list=PLxyz')).toBe(true);
	});

	it('returns true for video URL with list param', () => {
		expect(isPlaylistUrl('https://www.youtube.com/watch?v=abc&list=PLxyz')).toBe(true);
	});

	it('returns false for simple video URL', () => {
		expect(isPlaylistUrl('https://www.youtube.com/watch?v=abc123')).toBe(false);
	});
});
