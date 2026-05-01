import { derived } from 'svelte/store';
import { youtubeService } from '$services/youtube.service';
import type { MediaMode } from '$types/youtube.type';

function createMediaModeService() {
	const store = derived(youtubeService.store, ($s) => $s.mediaMode);

	function setMode(mode: MediaMode): void {
		youtubeService.setMediaMode(mode);
	}

	return { store, setMode };
}

export const mediaModeService = createMediaModeService();
export type { MediaMode };
