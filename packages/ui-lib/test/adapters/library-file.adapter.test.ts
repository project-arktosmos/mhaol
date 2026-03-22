import { describe, it, expect } from 'vitest';
import { libraryFileAdapter } from '../../src/adapters/classes/library-file.adapter';
import { MediaType } from '../../src/types/library.type';

describe('LibraryFileAdapter', () => {
	describe('formatSize', () => {
		it('formats zero bytes', () => {
			expect(libraryFileAdapter.formatSize(0)).toBe('0 B');
		});

		it('formats megabytes', () => {
			expect(libraryFileAdapter.formatSize(1048576)).toBe('1.0 MB');
		});
	});

	describe('getMediaTypeBadgeClass', () => {
		it('maps media types to badge classes', () => {
			expect(libraryFileAdapter.getMediaTypeBadgeClass(MediaType.Video)).toBe('badge-primary');
			expect(libraryFileAdapter.getMediaTypeBadgeClass(MediaType.Image)).toBe('badge-secondary');
			expect(libraryFileAdapter.getMediaTypeBadgeClass(MediaType.Audio)).toBe('badge-accent');
		});
	});

	describe('getMediaTypeLabel', () => {
		it('maps media types to labels', () => {
			expect(libraryFileAdapter.getMediaTypeLabel(MediaType.Video)).toBe('Video');
			expect(libraryFileAdapter.getMediaTypeLabel(MediaType.Image)).toBe('Image');
			expect(libraryFileAdapter.getMediaTypeLabel(MediaType.Audio)).toBe('Audio');
		});
	});

	describe('getCategoryBadgeClass', () => {
		it('maps categories to badge classes', () => {
			expect(libraryFileAdapter.getCategoryBadgeClass('tv')).toBe('badge-info');
			expect(libraryFileAdapter.getCategoryBadgeClass('movies')).toBe('badge-warning');
			expect(libraryFileAdapter.getCategoryBadgeClass('youtube')).toBe('badge-error');
			expect(libraryFileAdapter.getCategoryBadgeClass('uncategorized')).toBe('badge-ghost');
		});

		it('returns ghost for unknown category', () => {
			expect(libraryFileAdapter.getCategoryBadgeClass('other')).toBe('badge-ghost');
		});
	});

	describe('getCategoryLabel', () => {
		it('maps categories to labels', () => {
			expect(libraryFileAdapter.getCategoryLabel('tv')).toBe('TV');
			expect(libraryFileAdapter.getCategoryLabel('movies')).toBe('Movies');
		});

		it('returns raw ID for unknown category', () => {
			expect(libraryFileAdapter.getCategoryLabel('custom')).toBe('custom');
		});
	});
});
