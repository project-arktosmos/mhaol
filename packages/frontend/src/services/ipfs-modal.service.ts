import { writable } from 'svelte/store';

interface IpfsModalState {
	open: boolean;
}

function createIpfsModalService() {
	const store = writable<IpfsModalState>({ open: false });

	function open(): void {
		store.set({ open: true });
	}

	function close(): void {
		store.set({ open: false });
	}

	return { store, open, close };
}

export const ipfsModalService = createIpfsModalService();
