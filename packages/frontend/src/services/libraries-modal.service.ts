import { writable } from 'svelte/store';

interface LibrariesModalState {
	open: boolean;
}

function createLibrariesModalService() {
	const store = writable<LibrariesModalState>({ open: false });

	function open(): void {
		store.set({ open: true });
	}

	function close(): void {
		store.set({ open: false });
	}

	return { store, open, close };
}

export const librariesModalService = createLibrariesModalService();
