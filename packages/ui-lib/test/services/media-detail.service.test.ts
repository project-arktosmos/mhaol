import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { mediaDetailService } from '../../src/services/media-detail.service';

const mockSelection = {
	item: { id: '1', title: 'Test Movie' } as any,
	cardType: 'movie' as const,
	tmdbMetadata: null,
	youtubeMetadata: null,
	musicbrainzMetadata: null,
	imageTags: []
};

describe('mediaDetailService', () => {
	it('should start with null state', () => {
		expect(get(mediaDetailService.store)).toBeNull();
	});

	it('should select a media item', () => {
		mediaDetailService.select(mockSelection);
		const state = get(mediaDetailService.store);
		expect(state).not.toBeNull();
		expect(state!.cardType).toBe('movie');
		expect(state!.item.id).toBe('1');
	});

	it('should clear the selection', () => {
		mediaDetailService.select(mockSelection);
		mediaDetailService.clear();
		expect(get(mediaDetailService.store)).toBeNull();
	});

	it('should replace previous selection with new one', () => {
		mediaDetailService.select(mockSelection);
		const newSelection = {
			...mockSelection,
			item: { id: '2', title: 'Another Movie' } as any,
			cardType: 'tv' as const
		};
		mediaDetailService.select(newSelection);
		const state = get(mediaDetailService.store);
		expect(state!.cardType).toBe('tv');
		expect(state!.item.id).toBe('2');
	});
});
