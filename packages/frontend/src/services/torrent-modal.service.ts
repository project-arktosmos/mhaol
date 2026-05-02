import { writable } from 'svelte/store';

interface TorrentModalState {
	open: boolean;
}

function createTorrentModalService() {
	const store = writable<TorrentModalState>({ open: false });

	function open(): void {
		store.set({ open: true });
	}

	function close(): void {
		store.set({ open: false });
	}

	return { store, open, close };
}

export const torrentModalService = createTorrentModalService();
