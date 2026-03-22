import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { themeService } from '../../src/services/theme.service';

describe('themeService', () => {
	beforeEach(() => {
		document.documentElement.removeAttribute('data-theme');
	});

	it('should have light as default theme', () => {
		expect(themeService.currentTheme()).toBe('light');
	});

	it('should toggle from light to dark', () => {
		themeService.set({ id: 'theme-settings', theme: 'light' });
		themeService.toggle();
		expect(themeService.currentTheme()).toBe('dark');
	});

	it('should toggle from dark to light', () => {
		themeService.set({ id: 'theme-settings', theme: 'dark' });
		themeService.toggle();
		expect(themeService.currentTheme()).toBe('light');
	});

	it('should expose state via store', () => {
		themeService.set({ id: 'theme-settings', theme: 'dark' });
		const state = get(themeService.store);
		expect(state.theme).toBe('dark');
	});

	it('should set data-theme attribute on initialize', () => {
		themeService.set({ id: 'theme-settings', theme: 'dark' });
		themeService.initialize();
		expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
	});

	it('should set data-theme with variant prefix on initialize', () => {
		themeService.set({ id: 'theme-settings', theme: 'light' });
		themeService.initialize('mhaol');
		expect(document.documentElement.getAttribute('data-theme')).toBe('mhaol-light');
	});
});
