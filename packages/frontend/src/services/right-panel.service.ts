import { writable } from 'svelte/store';
import type { RightPanelVideo } from '$types/youtube.type';

interface RightPanelState {
	video: RightPanelVideo | null;
}

function createRightPanelService() {
	const store = writable<RightPanelState>({ video: null });

	function open(video: RightPanelVideo): void {
		store.set({ video });
	}

	function close(): void {
		store.set({ video: null });
	}

	return { store, open, close };
}

export const rightPanelService = createRightPanelService();
