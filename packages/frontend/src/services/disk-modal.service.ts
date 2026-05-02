import { writable } from 'svelte/store';

interface DiskModalState {
	open: boolean;
}

function createDiskModalService() {
	const store = writable<DiskModalState>({ open: false });

	function open(): void {
		store.set({ open: true });
	}

	function close(): void {
		store.set({ open: false });
	}

	return { store, open, close };
}

export const diskModalService = createDiskModalService();
