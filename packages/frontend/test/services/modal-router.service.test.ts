import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { modalRouterService } from '../../src/services/modal-router.service';

describe('modalRouterService', () => {
	beforeEach(() => {
		modalRouterService.closeAll();
		window.location.hash = '';
	});

	it('should start with null navbarModal and null mediaDetail', () => {
		const state = get(modalRouterService.store);
		expect(state.navbarModal).toBeNull();
		expect(state.mediaDetail).toBeNull();
	});

	it('should open a navbar modal', () => {
		modalRouterService.openNavbar('torrent');
		const state = get(modalRouterService.store);
		expect(state.navbarModal).toBe('torrent');
	});

	it('should close a navbar modal', () => {
		modalRouterService.openNavbar('torrent');
		modalRouterService.closeNavbar();
		const state = get(modalRouterService.store);
		expect(state.navbarModal).toBeNull();
	});

	it('should open a media detail', () => {
		modalRouterService.openMediaDetail('movie', 'popular', '42');
		const state = get(modalRouterService.store);
		expect(state.mediaDetail).toEqual({ type: 'movie', category: 'popular', id: '42' });
	});

	it('should close a media detail', () => {
		modalRouterService.openMediaDetail('movie', 'popular', '42');
		modalRouterService.closeMediaDetail();
		const state = get(modalRouterService.store);
		expect(state.mediaDetail).toBeNull();
	});

	it('should close all modals', () => {
		modalRouterService.openNavbar('torrent');
		modalRouterService.openMediaDetail('movie', 'popular', '42');
		modalRouterService.closeAll();
		const state = get(modalRouterService.store);
		expect(state.navbarModal).toBeNull();
		expect(state.mediaDetail).toBeNull();
	});

	it('should update URL hash when opening navbar modal', () => {
		modalRouterService.openNavbar('downloads');
		expect(window.location.hash).toBe('#downloads');
	});
});
