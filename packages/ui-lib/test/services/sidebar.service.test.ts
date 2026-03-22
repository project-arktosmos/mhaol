import { describe, it, expect } from 'vitest';
import { get } from 'svelte/store';
import { sidebarService } from '../../src/services/sidebar.service';

describe('sidebarService', () => {
	it('should have default width mode initially', () => {
		const state = get(sidebarService.store);
		expect(state.widthMode).toBe('default');
	});

	it('should set width mode to wide', () => {
		sidebarService.setWidthMode('wide');
		const state = get(sidebarService.store);
		expect(state.widthMode).toBe('wide');
	});

	it('should set width mode to narrow', () => {
		sidebarService.setWidthMode('narrow');
		const state = get(sidebarService.store);
		expect(state.widthMode).toBe('narrow');
	});

	it('should set width mode back to default', () => {
		sidebarService.setWidthMode('wide');
		sidebarService.setWidthMode('default');
		const state = get(sidebarService.store);
		expect(state.widthMode).toBe('default');
	});

	it('should have the correct id', () => {
		const state = get(sidebarService.store);
		expect(state.id).toBe('sidebar-settings');
	});
});
