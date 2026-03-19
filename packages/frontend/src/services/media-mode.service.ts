import { derived } from 'svelte/store';
import { youtubeService } from 'frontend/services/youtube.service';
import type { MediaMode } from 'frontend/types/youtube.type';

function createMediaModeService() {
	const store = derived(youtubeService.store, ($s) => $s.mediaMode);

	function setMode(mode: MediaMode): void {
		youtubeService.setMediaMode(mode);
	}

	return { store, setMode };
}

export const mediaModeService = createMediaModeService();
export type { MediaMode };
