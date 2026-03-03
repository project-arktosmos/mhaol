import { writable } from 'svelte/store';

function createLibraryModalService() {
	const store = writable<boolean>(false);

	return {
		store,
		open(): void {
			store.set(true);
		},
		close(): void {
			store.set(false);
		}
	};
}

export const libraryModalService = createLibraryModalService();
