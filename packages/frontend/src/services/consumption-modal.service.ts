import { writable } from 'svelte/store';

interface ConsumptionModalState {
	open: boolean;
}

function createConsumptionModalService() {
	const store = writable<ConsumptionModalState>({ open: false });

	function open(): void {
		store.set({ open: true });
	}

	function close(): void {
		store.set({ open: false });
	}

	return { store, open, close };
}

export const consumptionModalService = createConsumptionModalService();
