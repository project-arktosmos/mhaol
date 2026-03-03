import { writable } from 'svelte/store';

function createYoutubeModalService() {
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

export const youtubeModalService = createYoutubeModalService();
