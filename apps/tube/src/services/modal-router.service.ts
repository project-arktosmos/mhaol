import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import type { ModalRouterState } from '$types/modal.type';

function parseMediaDetail(search: string): ModalRouterState['mediaDetail'] {
	const params = new URLSearchParams(search);
	const type = params.get('type');
	const category = params.get('category');
	const id = params.get('id');
	if (type && category && id) return { type, category, id };
	return null;
}

function stateFromUrl(): ModalRouterState {
	if (!browser) return { mediaDetail: null };
	return {
		mediaDetail: parseMediaDetail(window.location.search)
	};
}

function buildUrl(search: string): string {
	const url = new URL(window.location.href);
	url.hash = '';
	url.search = search;
	return url.toString();
}

function createModalRouterService() {
	const store = writable<ModalRouterState>(stateFromUrl());

	if (browser) {
		window.addEventListener('popstate', () => {
			store.set(stateFromUrl());
		});
	}

	function openMediaDetail(type: string, category: string, id: string): void {
		store.update((s) => ({ ...s, mediaDetail: { type, category, id } }));
		const params = new URLSearchParams({ type, category, id });
		history.pushState(null, '', buildUrl(params.toString()));
	}

	function closeMediaDetail(): void {
		store.update((s) => ({ ...s, mediaDetail: null }));
		history.pushState(null, '', buildUrl(''));
	}

	function closeAll(): void {
		store.set({ mediaDetail: null });
		history.pushState(null, '', buildUrl(''));
	}

	return { store, openMediaDetail, closeMediaDetail, closeAll };
}

export const modalRouterService = createModalRouterService();
