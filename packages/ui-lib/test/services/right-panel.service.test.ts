import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { rightPanelService } from '../../src/services/right-panel.service';

const mockVideo = {
	videoId: 'abc123',
	title: 'Test Video',
	thumbnail: 'https://example.com/thumb.jpg',
	views: 1000,
	uploaderName: 'Test Channel'
};

describe('rightPanelService', () => {
	it('should start with null video', () => {
		const state = get(rightPanelService.store);
		expect(state.video).toBeNull();
	});

	it('should open with a video', () => {
		rightPanelService.open(mockVideo);
		const state = get(rightPanelService.store);
		expect(state.video).not.toBeNull();
		expect(state.video!.videoId).toBe('abc123');
		expect(state.video!.title).toBe('Test Video');
	});

	it('should close and set video to null', () => {
		rightPanelService.open(mockVideo);
		rightPanelService.close();
		const state = get(rightPanelService.store);
		expect(state.video).toBeNull();
	});

	it('should replace video when opening with a different one', () => {
		rightPanelService.open(mockVideo);
		const anotherVideo = { ...mockVideo, videoId: 'xyz789', title: 'Another Video' };
		rightPanelService.open(anotherVideo);
		const state = get(rightPanelService.store);
		expect(state.video!.videoId).toBe('xyz789');
	});
});
