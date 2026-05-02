import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import type { ModalRouterState } from '$types/modal.type';

function parseHash(hash: string): string | null {
	const id = hash.replace('#', '');
	return id || null;
}

function parseMediaDetail(search: string): ModalRouterState['mediaDetail'] {
	const params = new URLSearchParams(search);
	const type = params.get('type');
	const category = params.get('category');
	const id = params.get('id');
	if (type && category && id) return { type, category, id };
	return null;
}

function stateFromUrl(): ModalRouterState {
	if (!browser) return { navbarModal: null, mediaDetail: null };
	return {
		navbarModal: parseHash(window.location.hash),
		mediaDetail: parseMediaDetail(window.location.search)
	};
}

function buildUrl(hash: string, search: string): string {
	const url = new URL(window.location.href);
	url.hash = hash;
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

	function openNavbar(id: string): void {
		store.update((s) => ({ ...s, navbarModal: id }));
		history.pushState(null, '', buildUrl(`#${id}`, ''));
	}

	function closeNavbar(): void {
		store.update((s) => ({ ...s, navbarModal: null }));
		history.pushState(null, '', buildUrl('', window.location.search));
	}

	function openMediaDetail(type: string, category: string, id: string): void {
		store.update((s) => ({ ...s, mediaDetail: { type, category, id } }));
		const params = new URLSearchParams({ type, category, id });
		history.pushState(null, '', buildUrl('', params.toString()));
	}

	function closeMediaDetail(): void {
		store.update((s) => ({ ...s, mediaDetail: null }));
		history.pushState(null, '', buildUrl(window.location.hash, ''));
	}

	function closeAll(): void {
		store.set({ navbarModal: null, mediaDetail: null });
		history.pushState(null, '', buildUrl('', ''));
	}

	return { store, openNavbar, closeNavbar, openMediaDetail, closeMediaDetail, closeAll };
}

export const modalRouterService = createModalRouterService();
