import { writable } from 'svelte/store';

interface ArtistsModalState {
	open: boolean;
}

function createArtistsModalService() {
	const store = writable<ArtistsModalState>({ open: false });

	function open(): void {
		store.set({ open: true });
	}

	function close(): void {
		store.set({ open: false });
	}

	return { store, open, close };
}

export const artistsModalService = createArtistsModalService();
