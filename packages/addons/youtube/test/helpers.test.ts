import { describe, it, expect } from 'vitest';
import { getThumbnailUrl } from '../src/helpers';

describe('getThumbnailUrl', () => {
	it('returns correct YouTube thumbnail URL', () => {
		expect(getThumbnailUrl('dQw4w9WgXcQ')).toBe(
			'https://img.youtube.com/vi/dQw4w9WgXcQ/hqdefault.jpg'
		);
	});

	it('handles empty video ID', () => {
		expect(getThumbnailUrl('')).toBe('https://img.youtube.com/vi//hqdefault.jpg');
	});
});
