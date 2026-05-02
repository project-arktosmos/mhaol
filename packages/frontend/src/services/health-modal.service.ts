import { writable } from 'svelte/store';

interface HealthModalState {
	open: boolean;
}

function createHealthModalService() {
	const store = writable<HealthModalState>({ open: false });

	function open(): void {
		store.set({ open: true });
	}

	function close(): void {
		store.set({ open: false });
	}

	return { store, open, close };
}

export const healthModalService = createHealthModalService();
