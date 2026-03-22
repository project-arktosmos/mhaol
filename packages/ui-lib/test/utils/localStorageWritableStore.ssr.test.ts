import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

// Mock $app/environment with browser = false BEFORE importing the module
vi.mock('$app/environment', () => ({
	browser: false,
	building: false,
	dev: true,
	version: 'test'
}));

// Import after mocking
import localStorageWritableStore from '../../src/utils/localStorageWritableStore';

describe('localStorageWritableStore (SSR mode)', () => {
	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('should return a regular writable store when browser is false', () => {
		const store = localStorageWritableStore('ssr-key', 'initial-value');

		expect(get(store)).toBe('initial-value');
	});

	it('should allow setting values without localStorage', () => {
		const store = localStorageWritableStore('ssr-key', { id: 1, name: 'Test' });

		store.set({ id: 2, name: 'Updated' });

		expect(get(store)).toEqual({ id: 2, name: 'Updated' });
	});

	it('should allow updating values without localStorage', () => {
		const store = localStorageWritableStore('ssr-key', 0);

		store.update((n) => n + 1);

		expect(get(store)).toBe(1);
	});

	it('should support subscriptions in SSR mode', () => {
		const store = localStorageWritableStore('ssr-key', 'initial');
		const values: string[] = [];

		const unsubscribe = store.subscribe((value) => values.push(value));

		store.set('updated');

		expect(values).toEqual(['initial', 'updated']);

		unsubscribe();
	});

	it('should work with complex objects in SSR mode', () => {
		const complexValue = {
			user: { id: 1, profile: { name: 'John' } },
			items: [1, 2, 3]
		};

		const store = localStorageWritableStore('ssr-complex', complexValue);

		expect(get(store)).toEqual(complexValue);

		store.update((val) => ({
			...val,
			items: [...val.items, 4]
		}));

		expect(get(store).items).toEqual([1, 2, 3, 4]);
	});
});
