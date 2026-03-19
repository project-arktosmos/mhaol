import { writable } from 'svelte/store';
import type { MediaDetailSelection } from '$types/media-detail.type';

function createMediaDetailService() {
	const store = writable<MediaDetailSelection | null>(null);

	return {
		store,
		select(selection: MediaDetailSelection): void {
			store.set(selection);
		},
		clear(): void {
			store.set(null);
		}
	};
}

export const mediaDetailService = createMediaDetailService();
