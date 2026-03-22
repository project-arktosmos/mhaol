import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { mediaModeService } from '../../src/services/media-mode.service';
import { youtubeService } from '../../src/services/youtube.service';

describe('mediaModeService', () => {
	it('should expose the current media mode from youtube service', () => {
		// mediaMode is optional in settings, so it may be undefined initially
		const mode = get(mediaModeService.store);
		expect(mode === undefined || mode === 'audio' || mode === 'video').toBe(true);
	});

	it('should change mode to video', () => {
		mediaModeService.setMode('video');
		const mode = get(mediaModeService.store);
		expect(mode).toBe('video');
	});

	it('should change mode to audio', () => {
		mediaModeService.setMode('audio');
		const mode = get(mediaModeService.store);
		expect(mode).toBe('audio');
	});

	it('should reflect changes made via youtubeService', () => {
		youtubeService.setMediaMode('video');
		expect(get(mediaModeService.store)).toBe('video');
		youtubeService.setMediaMode('audio');
		expect(get(mediaModeService.store)).toBe('audio');
	});
});
